/*!
 * CGI module
 *
 * Common Gateway Interface implementation
 */

pub mod executor;
pub mod environment;

pub use executor::CgiExecutor;
pub use environment::CgiEnvironment;
