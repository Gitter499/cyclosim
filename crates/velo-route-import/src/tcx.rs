use crate::ImportError;
use velo_core::route::haversine_m;
use velo_core::RoutePoint;

/// Basic TCX trackpoint parser (Training Center XML).
pub fn parse_tcx_stub(data: &[u8]) -> Result<Vec<RoutePoint>, ImportError> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_reader(data);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut points = Vec::new();
    let mut in_tp = false;
    let mut lat = None::<f64>;
    let mut lon = None::<f64>;
    let mut ele = None::<f64>;
    let mut field = String::new();
    let mut cumulative = 0.0;
    let mut prev = None::<(f64, f64)>;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let local_name = e.name().local_name();
                let local = String::from_utf8_lossy(local_name.as_ref());
                if local == "Trackpoint" {
                    in_tp = true;
                    lat = None;
                    lon = None;
                    ele = None;
                }
                field = local.to_string();
            }
            Ok(Event::Text(t)) if in_tp => {
                let text = t.unescape().unwrap_or_default();
                match field.as_str() {
                    "LatitudeDegrees" => lat = text.parse().ok(),
                    "LongitudeDegrees" => lon = text.parse().ok(),
                    "AltitudeMeters" => ele = text.parse().ok(),
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                if e.name().local_name().as_ref() == b"Trackpoint" {
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
                    in_tp = false;
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
