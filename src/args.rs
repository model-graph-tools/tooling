use clap::ArgMatches;
use wildfly_container_versions::WildFlyContainer;

pub fn wildfly_container_argument(matches: &ArgMatches) -> WildFlyContainer {
    matches
        .get_one::<WildFlyContainer>("identifier")
        .expect("Argument <identifier> expected!")
        .clone()
}

pub fn wildfly_containers_argument(matches: &ArgMatches) -> Vec<WildFlyContainer> {
    matches
        .get_one::<Vec<WildFlyContainer>>("identifier")
        .expect("Argument <identifier> expected!")
        .clone()
}
