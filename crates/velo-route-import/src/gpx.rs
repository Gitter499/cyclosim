use quick_xml::events::Event;
use quick_xml::Reader;

use crate::ImportError;
use velo_core::route::haversine_m;
use velo_core::RoutePoint;

/// Parse GPX 1.0/1.1 track or route points (lat/lon + optional ele).
pub fn parse_gpx(data: &[u8]) -> Result<Vec<RoutePoint>, ImportError> {
    let mut reader = Reader::from_reader(data);
    reader.config_mut().trim_text(true);

    let mut points = Vec::new();
    let mut buf = Vec::new();
    let mut in_trkpt = false;
    let mut in_rtept = false;
    let mut lat: Option<f64> = None;
    let mut lon: Option<f64> = None;
    let mut ele: Option<f64> = None;
    let mut cumulative = 0.0;
    let mut prev: Option<(f64, f64)> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let local_name = e.name().local_name();
                let local = String::from_utf8_lossy(local_name.as_ref());
                match local.as_ref() {
                    "trkpt" | "rtept" => {
                        if local == "trkpt" {
                            in_trkpt = true;
                        } else {
                            in_rtept = true;
                        }
                        lat = e
                            .attributes()
                            .find(|a| a.as_ref().map(|a| a.key.as_ref()) == Ok(b"lat"))
                            .and_then(|a| a.ok())
                            .and_then(|a| String::from_utf8_lossy(&a.value).parse().ok());
                        lon = e
                            .attributes()
                            .find(|a| a.as_ref().map(|a| a.key.as_ref()) == Ok(b"lon"))
                            .and_then(|a| a.ok())
                            .and_then(|a| String::from_utf8_lossy(&a.value).parse().ok());
                        ele = None;
                    }
                    "ele" if in_trkpt || in_rtept => {}
                    _ => {}
                }
            }
            Ok(Event::Text(t)) if in_trkpt || in_rtept => {
                let text = t.unescape().unwrap_or_default();
                if let Ok(v) = text.parse::<f64>() {
                    ele = Some(v);
                }
            }
            Ok(Event::End(e)) => {
                let local_name = e.name().local_name();
                let local = String::from_utf8_lossy(local_name.as_ref());
                if local == "trkpt" || local == "rtept" {
                    if let (Some(la), Some(lo)) = (lat, lon) {
                        if let Some((pla, plo)) = prev {
                            cumulative += haversine_m(pla, plo, la, lo);
                        }
                        points.push(RoutePoint {
                            distance_m: cumulative,
                            lat: la,
                            lon: lo,
                            elevation_m: ele.unwrap_or(0.0),
                            grade: 0.0,
                        });
                        prev = Some((la, lo));
                    }
                    in_trkpt = false;
                    in_rtept = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(ImportError::Xml(e.to_string())),
            _ => {}
        }
        buf.clear();
    }

    Ok(points)
}
