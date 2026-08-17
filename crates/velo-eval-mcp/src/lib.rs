//! VeloSim evaluation library: deterministic scenarios, headless frame
//! rendering, and the MCP tool registry. The `velo-eval-mcp` binary wraps
//! this behind an MCP stdio server and a one-shot CLI.

pub mod frames;
pub mod mcp;
pub mod scenario;
pub mod tools;
