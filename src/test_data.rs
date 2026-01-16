use crate::dao;
use dao::repeater::NewRepeater;

pub fn repeaters() -> Vec<NewRepeater> {
    Vec::from([
        // LA4O
        NewRepeater::fm_narrow("LA5OR", 145_600_000, -600_000)
            .ctcss(123.0)
            .maidenhead_locator("JO59ix")
            .address("Tryvann, Oslo"),
        NewRepeater::fm_narrow("LA7OR", 434_775_000, -2_000_000)
            .address("Brannfjell, Oslo")
            .maidenhead_locator("JO59jv"),
        NewRepeater::dmr("LD1OA", 434_887_500, -2_000_000, 242002)
            .address("Røverkollen, Oslo")
            .maidenhead_locator("JO59kx"),
        // TODO: Add C4FM entry
    ])
}
