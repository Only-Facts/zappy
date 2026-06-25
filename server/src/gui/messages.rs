use crate::entities::egg::Egg;
use crate::entities::player::Player;
use crate::world::map::{GameMap, Resource};
use std::fmt::Write;

pub fn msz(width: usize, height: usize) -> String {
    format!("msz {width} {height}\n")
}

pub fn bct(map: &GameMap, x: usize, y: usize) -> String {
    let Some(tile) = map.get_tile(x, y) else {
        return String::new();
    };

    let quantities = tile.quantities();

    format!(
        "bct {x} {y} {} {} {} {} {} {} {}\n",
        quantities[0],
        quantities[1],
        quantities[2],
        quantities[3],
        quantities[4],
        quantities[5],
        quantities[6],
    )
}

pub fn tna(team_name: &str) -> String {
    format!("tna {team_name}\n")
}

pub fn pnw(player: &Player) -> String {
    format!(
        "pnw #{} {} {} {} {} {}\n",
        player.id,
        player.x,
        player.y,
        player.orientation.gui_value(),
        player.level,
        player.team_name
    )
}

pub fn ppo(player: &Player) -> String {
    format!(
        "ppo #{} {} {} {}\n",
        player.id,
        player.x,
        player.y,
        player.orientation.gui_value()
    )
}

pub fn plv(player: &Player) -> String {
    format!("plv #{} {}\n", player.id, player.level)
}

pub fn pin(player: &Player) -> String {
    let quantities = player.inventory.quantities();

    format!(
        "pin #{} {} {} {} {} {} {} {} {} {}\n",
        player.id,
        player.x,
        player.y,
        quantities[0],
        quantities[1],
        quantities[2],
        quantities[3],
        quantities[4],
        quantities[5],
        quantities[6],
    )
}

pub fn pex(player_id: usize) -> String {
    format!("pex #{player_id}\n")
}

pub fn pbc(player_id: usize, message: &str) -> String {
    format!("pbc #{player_id} {message}\n")
}

pub fn pic(x: usize, y: usize, level: usize, participants: &[usize]) -> String {
    let mut player_ids = String::new();

    for player_id in participants {
        let _ = write!(player_ids, " #{player_id}");
    }

    format!("pic {x} {y} {level}{player_ids}\n")
}

pub fn pie(x: usize, y: usize, success: bool) -> String {
    format!("pie {x} {y} {}\n", usize::from(success))
}

pub fn pfk(player_id: usize) -> String {
    format!("pfk #{player_id}\n")
}

pub fn pdr(player_id: usize, resource: Resource) -> String {
    format!("pdr #{player_id} {}\n", resource.gui_index())
}

pub fn pgt(player_id: usize, resource: Resource) -> String {
    format!("pgt #{player_id} {}\n", resource.gui_index())
}

pub fn pdi(player_id: usize) -> String {
    format!("pdi #{player_id}\n")
}

pub fn enw(egg: &Egg) -> String {
    let parent_id = egg
        .parent_id
        .map(|player_id| format!("#{player_id}"))
        .unwrap_or_else(|| "#-1".to_string());

    format!("enw #{} {parent_id} {} {}\n", egg.id, egg.x, egg.y)
}

pub fn ebo(egg_id: usize) -> String {
    format!("ebo #{egg_id}\n")
}

pub fn edi(egg_id: usize) -> String {
    format!("edi #{egg_id}\n")
}

pub fn sgt(frequency: usize) -> String {
    format!("sgt {frequency}\n")
}

pub fn seg(team_name: &str) -> String {
    format!("seg {team_name}\n")
}
