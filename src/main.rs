//! Computes the probability of beating various DCs in Dimension 20's Never Stop Blowing Up.

use tabled::{builder::Builder, settings::style::Style};

/// Kinds of dice available in Never Stop Blowing Up.
#[derive(Debug, Copy, Clone)]
enum Die {
    D4,
    D6,
    D8,
    D10,
    D12,
    D20,
}

impl std::fmt::Display for Die {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Die::D4 => "d4",
            Die::D6 => "d6",
            Die::D8 => "d8",
            Die::D10 => "d10",
            Die::D12 => "d12",
            Die::D20 => "d20",
        };
        write!(f, "{}", s)
    }
}

impl Die {
    /// Returns the number of sides on the die.
    fn sides(self) -> u32 {
        match self {
            Die::D4 => 4,
            Die::D6 => 6,
            Die::D8 => 8,
            Die::D10 => 10,
            Die::D12 => 12,
            Die::D20 => 20,
        }
    }

    /// Returns the next highest die type, saturating at a d20.
    fn next(self) -> Die {
        match self {
            Die::D4 => Die::D6,
            Die::D6 => Die::D8,
            Die::D8 => Die::D10,
            Die::D10 => Die::D12,
            Die::D12 => Die::D20,
            Die::D20 => Die::D20,
        }
    }
}

/// Computes the probability of beating a given difficulty class when starting with the given die
/// type. Turbo tokens are not considered in this function.
///
/// In Never Stop Blowing Up, each player begins with each of their skills at a d4. When making a
/// check, the player rolls the die associated with the skill. If the result is the maximum value
/// for the die, the die explodes, and the player upgrades to the next highest die type and rolls
/// again, adding the result to the maximum of the previous die. This explosion process can
/// continue up to a d20, at which point the dice cannot explode any further, and the player will
/// reroll the d20 until they no longer roll the maximum value.
///
/// # Arguments
///
/// * `die` - The type of die being rolled.
/// * `dc` - The difficulty class to beat.
fn probability_of_success(die: Die, dc: u32) -> f64 {
    // Can always roll a 1 or higher.
    if dc <= 1 {
        return 1.0;
    }

    // If the DC is lower than or equal to the maximum value of the die.
    if dc <= die.sides() {
        return (die.sides() - dc + 1) as f64 // # of successful outcomes
            / die.sides() as f64; // # of total outcomes
    }

    // If the DC is higher than the maximum value of the die, explode the die and recurse.
    let p = 1.0 / die.sides() as f64; // Probability of exploding.
    p * probability_of_success(die.next(), dc - die.sides())
}

/// Computes the probability of beating a given difficulty class when starting with the given die
/// type and using available turbo tokens to explode the die.
///
/// When a player fails an ability check, they receieve a turbo token. Turbo tokens can be spent
/// on any later roll to add 1 to the result of their roll for each token spent. Players can use
/// this mechanic to explode their die if they have enough turbo tokens to increase their roll to
/// the max value of the die.
///
/// This function assumes that a player will always spend a turbo token to explode the die if they
/// can.
///
/// # Arguments
///
/// * `die` - The type of die being rolled.
/// * `turbo_tokens` - The number of turbo tokens available to the player.
/// * `dc` - The difficulty class to beat.
fn probability_of_success_with_turbo_tokens(die: Die, turbo_tokens: u32, dc: u32) -> f64 {
    // Can always roll a 1 or higher.
    if dc <= 1 {
        return 1.0;
    }

    // If the DC is lower than or equal to the maximum value of the die.
    if dc <= die.sides() {
        // Turbo token can be counted as a successful outcome.
        return (die.sides() - dc + 1 + turbo_tokens).min(die.sides()) as f64 // # of successful outcomes
            / die.sides() as f64; // # of total outcomes
    }

    // If the DC is higher than the maximum value of the die, explode the die and recurse.
    (1..=die.sides())
        .map(|roll| { // Consider all possible rolls with the current die.
            if roll + turbo_tokens < die.sides() - 1 {
                // The die cannot explode, even with using all turbo tokens.
                return 0.0;
            }

            // Die will explode (it must for a chance to beat the DC).
            let tokens_needed_to_explode = (die.sides() - roll).saturating_sub(1);
            probability_of_success_with_turbo_tokens(die.next(), turbo_tokens - tokens_needed_to_explode, dc - roll)
                / die.sides() as f64
        })
        .sum()
}

fn main() {
    // Generate table of probabilities for each DC and die type.
    let max_dc = 80;
    let max_turbo_tokens = 5;
    let dice = [Die::D4, Die::D6, Die::D8, Die::D10, Die::D12, Die::D20];
    let style = Style::markdown();

    for turbo_tokens in 0..=max_turbo_tokens {
        let mut table = Builder::default();

        let header = std::iter::once("DC".to_string())
            .chain(dice.iter().map(|die| die.to_string()));
        table.push_record(header);

        (1..=max_dc)
            .map(|dc| {
                let probabilties = dice
                    .into_iter()
                    .map(move |die| probability_of_success_with_turbo_tokens(die, turbo_tokens, dc))
                    .map(|p| format!("{:.6}%", p * 100.0));
                std::iter::once(dc.to_string()).chain(probabilties)
            })
            .for_each(|record| table.push_record(record));

        let mut table = table.build();
        table.with(style.clone());

        println!("## {} turbo tokens", turbo_tokens);
        println!();
        println!("{}", table);
        println!();
    }
}
