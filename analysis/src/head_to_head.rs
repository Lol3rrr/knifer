use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct Output {
    pub players: HashMap<csdemo::UserId, csdemo::parser::Player>,
    pub head_to_head: HashMap<csdemo::UserId, HashMap<csdemo::UserId, usize>>,
}

pub fn parse(buf: &[u8]) -> Result<Output, ()> {
    let tmp = csdemo::Container::parse(buf).map_err(|e| ())?;

    let output = csdemo::lazyparser::LazyParser::new(tmp);

    let players = output.player_info();

    let mut head_to_head = HashMap::new();

    for event in output.events().filter_map(|e| e.ok()) {
        let event = match event {
            csdemo::DemoEvent::GameEvent(ge) => *ge,
            _ => continue,
        };

        match event {
            csdemo::game_event::GameEvent::PlayerDeath(death) => {
                let (attacker_player, attacker) = match death.attacker.and_then(|u| players.get(&u).zip(Some(u))) {
                    Some(a) => a,
                    None => continue,
                };

                let (died_player, died) = match death.userid.and_then(|u| players.get(&u).zip(Some(u))) {
                    Some(d) => d,
                    None => continue,
                };

                if attacker_player.team == died_player.team {
                    continue;
                }

                let attacker_entry: &mut HashMap<_, _> = head_to_head.entry(attacker).or_default();
                let died_killed: &mut usize = attacker_entry.entry(died).or_default();
                *died_killed += 1;
            }
            _ => {}
        };
    }

    Ok(Output {
        players,
        head_to_head,
    })
}
