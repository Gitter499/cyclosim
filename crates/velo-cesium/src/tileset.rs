use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TilesetError {
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("empty tileset")]
    Empty,
    #[error("no root tile")]
    NoRoot,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TilesetDocument {
    pub asset: TilesetAsset,
    #[serde(default)]
    pub geometric_error: f64,
    pub root: TileNode,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TilesetAsset {
    pub version: String,
    #[serde(default)]
    pub tileset_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TileNode {
    #[serde(default)]
    pub geometric_error: f64,
    #[serde(default)]
    pub refine: Option<String>,
    #[serde(default)]
    pub content: Option<TileContent>,
    #[serde(default)]
    pub children: Vec<TileNode>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TileContent {
    #[serde(default)]
    pub uri: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

impl TileContent {
    pub fn href(&self) -> Option<&str> {
        self.uri.as_deref().or(self.url.as_deref())
    }
}

impl TilesetDocument {
    pub fn parse_json(json: &str) -> Result<Self, TilesetError> {
        let doc: Self = serde_json::from_str(json)?;
        if doc.asset.version != "1.0" && doc.asset.version != "1.1" {
            // 3D Tiles uses asset.version "1.0" or "1.1"
        }
        Ok(doc)
    }

    /// Collect content URIs reachable from root (breadth-first, depth-limited).
    pub fn content_uris(&self, max_depth: u32) -> Vec<String> {
        let mut out = Vec::new();
        let mut queue: Vec<(&TileNode, u32)> = vec![(&self.root, 0)];
        while let Some((node, depth)) = queue.pop() {
            if let Some(content) = &node.content {
                if let Some(href) = content.href() {
                    out.push(href.to_string());
                }
            }
            if depth < max_depth {
                for child in &node.children {
                    queue.push((child, depth + 1));
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "asset": { "version": "1.0" },
        "geometricError": 500,
        "root": {
            "geometricError": 100,
            "refine": "ADD",
            "content": { "uri": "tile.glb" },
            "children": [
                { "geometricError": 0, "content": { "uri": "child/tile.glb" } }
            ]
        }
    }"#;

    #[test]
    fn parse_tileset() {
        let doc = TilesetDocument::parse_json(SAMPLE).unwrap();
        assert_eq!(doc.asset.version, "1.0");
    }

    #[test]
    fn collect_uris() {
        let doc = TilesetDocument::parse_json(SAMPLE).unwrap();
        let uris = doc.content_uris(2);
        assert_eq!(uris.len(), 2);
        assert!(uris.contains(&"tile.glb".to_string()));
    }
}
