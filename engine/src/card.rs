use std::collections::HashMap;
use std::any::Any;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CardType 
{
    Land,
    Creature,
}

// Use composition so only creatures have power/toughness.
#[derive(Copy, Clone, Debug)]
pub struct CreatureStats
{
    pub power: u8,
    pub toughness: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CardFragmentKind
{
    Creature,
}

pub trait Fragment: Any + Send + Sync
{
    fn as_any(&self) -> &dyn Any;
    fn box_clone(&self) -> Box<dyn Fragment>;
}

#[derive(Clone, Debug)]
pub struct CreatureFragment
{
    pub stats: CreatureStats,
}

impl Fragment for CreatureFragment
{
    fn as_any(&self) -> &dyn Any
    {
        self
    }

    fn box_clone(&self) -> Box<dyn Fragment>
    {
        Box::new(CreatureFragment { stats: self.stats })
    }
}

impl Clone for Box<dyn Fragment>
{
    fn clone(&self) -> Box<dyn Fragment>
    {
        self.box_clone()
    }
}

#[derive(Clone)]
pub struct Card
{
    pub name: &'static str,
    pub card_types: Vec<CardType>,
    pub cost: u32,
    pub fragments: HashMap<CardFragmentKind, Box<dyn Fragment>>,
}

impl Card
{
    pub fn is_type(&self, t: CardType) -> bool
    {
        self.card_types.iter().any(|ct| *ct == t)
    }

    pub fn add_type(&mut self, t: CardType)
    {
        if !self.card_types.contains(&t)
        {
            self.card_types.push(t);
        }
    }

    pub fn remove_type(&mut self, t: CardType)
    {
        if let Some(pos) = self.card_types.iter().position(|ct| *ct == t)
        {
            self.card_types.remove(pos);
        }
    }
}

#[derive(Clone)]
pub struct Deck
{
    pub cards: Vec<Card>,
}

impl Deck
{
    pub fn count(&self, card_type: CardType) -> usize 
    {
        self.cards.iter().filter(|c| c.is_type(card_type)).count()
    }
}

pub fn forest() -> Card 
{
    Card
    {
        name: "Forest",
        card_types: vec![CardType::Land],
        cost: 0,
        fragments: HashMap::new(),
    }
}

pub fn grizzly_bears() -> Card 
{
    Card
    {
        name: "Grizzly Bears",
        card_types: vec![CardType::Creature],
        cost: 2,
        fragments: {
            let mut m = HashMap::new();
            m.insert(
                CardFragmentKind::Creature,
                Box::new(CreatureFragment { stats: CreatureStats { power: 2, toughness: 2 } }) as Box<dyn Fragment>,
            );
            m
        },
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::creature;

    #[test]
    fn card_composition_and_type_mutation()
    {
        let f = forest();
        assert!(!creature::is_creature(&f));
        assert!(creature::creature_stats(&f).is_none());

        let mut g = grizzly_bears();
        assert!(creature::is_creature(&g));
        assert!(creature::creature_stats(&g).is_some());
        assert_eq!(creature::creature_stats(&g).unwrap().power, 2);

        // remove creature type (doesn't automatically remove fragment)
        g.remove_type(CardType::Creature);
        assert!(!g.is_type(CardType::Creature));

        // fragment still present until explicitly removed
        creature::remove_creature_fragment(&mut g);
        assert!(!creature::is_creature(&g));
        assert!(creature::creature_stats(&g).is_none());

        // add creature type back and set creature fragment
        g.add_type(CardType::Creature);
        creature::add_creature_fragment(&mut g, 3, 3);
        assert!(creature::is_creature(&g));
        assert_eq!(creature::creature_stats(&g).unwrap().power, 3);
    }
}
