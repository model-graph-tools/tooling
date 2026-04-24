use clap::ArgMatches;
use wildfly_container_versions::WildFlyContainer;

pub fn wildfly_container_argument(matches: &ArgMatches) -> WildFlyContainer {
    matches
        .get_one::<WildFlyContainer>("identifier")
        .expect("Argument <identifier> expected!")
        .clone()
}
