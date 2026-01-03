use crate::card::{Card, CardType, CardFragmentKind, CreatureFragment, CreatureStats};

pub fn is_creature(card: &Card) -> bool
{
    card.card_types.iter().any(|ct| *ct == CardType::Creature)
        || card.fragments.contains_key(&CardFragmentKind::Creature)
}

pub fn creature_stats(card: &Card) -> Option<CreatureStats>
{
    card.fragments.get(&CardFragmentKind::Creature).and_then(|f|
        f.as_any().downcast_ref::<CreatureFragment>().map(|cf| cf.stats)
    )
}

pub fn add_creature_fragment(card: &mut Card, power: u8, toughness: u8)
{
    card.fragments.insert(
        CardFragmentKind::Creature,
        Box::new(CreatureFragment { stats: CreatureStats { power, toughness } }),
    );
}

pub fn remove_creature_fragment(card: &mut Card)
{
    card.fragments.remove(&CardFragmentKind::Creature);
}
