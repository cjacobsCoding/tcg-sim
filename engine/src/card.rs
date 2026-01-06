use std::collections::HashMap;
use std::any::Any;
use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardType 
{
    Land,
    Creature,
}

// Use composition so only creatures have power/toughness.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct CreatureStats
{
    pub power: u8,
    pub toughness: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardFragmentKind
{
    Creature,
    Tappable,
}

pub trait Fragment: Any + Send + Sync
{
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn box_clone(&self) -> Box<dyn Fragment>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreatureFragment
{
    pub stats: CreatureStats,
    pub summoning_sickness: bool,
}

impl Fragment for CreatureFragment
{
    fn as_any(&self) -> &dyn Any
    {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any
    {
        self
    }

    fn box_clone(&self) -> Box<dyn Fragment>
    {
        Box::new(CreatureFragment { stats: self.stats, summoning_sickness: self.summoning_sickness })
    }
}

impl Fragment for TappableFragment
{
    fn as_any(&self) -> &dyn Any
    {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any
    {
        self
    }

    fn box_clone(&self) -> Box<dyn Fragment>
    {
        Box::new(TappableFragment { tapped: self.tapped })
    }
}

impl Clone for Box<dyn Fragment>
{
    fn clone(&self) -> Box<dyn Fragment>
    {
        self.box_clone()
    }
}

// Serializable representation of fragments
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SerializableFragment
{
    Creature(CreatureFragment),
    Tappable(TappableFragment),
}

impl SerializableFragment
{
    /// Convert to trait object
    pub fn to_fragment(&self) -> Box<dyn Fragment>
    {
        match self
        {
            SerializableFragment::Creature(cf) => Box::new(cf.clone()),
            SerializableFragment::Tappable(tf) => Box::new(tf.clone()),
        }
    }

    /// Convert from trait object (best effort)
    pub fn from_fragment(fragment: &dyn Fragment) -> Option<Self>
    {
        if let Some(cf) = fragment.as_any().downcast_ref::<CreatureFragment>()
        {
            return Some(SerializableFragment::Creature(cf.clone()));
        }
        if let Some(tf) = fragment.as_any().downcast_ref::<TappableFragment>()
        {
            return Some(SerializableFragment::Tappable(tf.clone()));
        }
        None
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TappableFragment
{
    pub tapped: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Card
{
    pub name: String,
    pub card_types: Vec<CardType>,
    pub cost: u32,
    #[serde(serialize_with = "serialize_fragments", deserialize_with = "deserialize_fragments")]
    pub fragments: HashMap<CardFragmentKind, Box<dyn Fragment>>,
}

// Custom serialization for fragments
fn serialize_fragments<S>(
    fragments: &HashMap<CardFragmentKind, Box<dyn Fragment>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let serializable: HashMap<CardFragmentKind, SerializableFragment> = fragments
        .iter()
        .filter_map(|(k, v)| {
            SerializableFragment::from_fragment(v.as_ref()).map(|sf| (*k, sf))
        })
        .collect();
    serializable.serialize(serializer)
}

// Custom deserialization for fragments
fn deserialize_fragments<'de, D>(
    deserializer: D,
) -> Result<HashMap<CardFragmentKind, Box<dyn Fragment>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let serializable: HashMap<CardFragmentKind, SerializableFragment> =
        HashMap::deserialize(deserializer)?;
    Ok(serializable
        .into_iter()
        .map(|(k, v)| (k, v.to_fragment()))
        .collect())
}

impl std::fmt::Debug for Card
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("Card")
            .field("name", &self.name)
            .field("card_types", &self.card_types)
            .field("cost", &self.cost)
            .finish()
    }
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

    pub fn example() -> Deck
    {
        let mut cards = Vec::new();
        // default example deck: 29 lands, 31 nonlands (grizzly bears)
        for _ in 0..29 {
            cards.push(forest());
        }
        for _ in 0..31 {
            cards.push(grizzly_bears());
        }

        Deck { cards }
    }
}

pub fn forest() -> Card 
{
    Card
    {
        name: String::from("Forest"),
        card_types: vec![CardType::Land],
        cost: 0,
        fragments: {
            let mut m = HashMap::new();
            m.insert(
                CardFragmentKind::Tappable,
                Box::new(TappableFragment { tapped: false }) as Box<dyn Fragment>,
            );
            m
        },
    }
}

pub fn grizzly_bears() -> Card 
{
    Card
    {
        name: String::from("Grizzly Bears"),
        card_types: vec![CardType::Creature],
        cost: 2,
        fragments: {
            let mut m = HashMap::new();
            m.insert(
                CardFragmentKind::Creature,
                Box::new(CreatureFragment { stats: CreatureStats { power: 2, toughness: 2 }, summoning_sickness: false }) as Box<dyn Fragment>,
            );
            m.insert(
                CardFragmentKind::Tappable,
                Box::new(TappableFragment { tapped: false }) as Box<dyn Fragment>,
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
