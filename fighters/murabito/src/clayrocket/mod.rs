use super::*;

mod acmd;
mod opff;

pub fn install() {
    let agent = &mut Agent::new("murabito_clayrocket");
    acmd::install(agent);
    opff::install(agent);
    agent.install();
}