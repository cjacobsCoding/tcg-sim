use crate::card::{Card, CardFragmentKind, TappableFragment};

pub fn is_tappable(card: &Card) -> bool
{
    card.fragments.contains_key(&CardFragmentKind::Tappable)
}

pub fn is_tapped(card: &Card) -> bool
{
    card.fragments.get(&CardFragmentKind::Tappable)
        .and_then(|f| f.as_any().downcast_ref::<TappableFragment>().map(|tf| tf.tapped))
        .unwrap_or(false)
}

pub fn set_tapped(card: &mut Card, value: bool)
{
    if let Some(f) = card.fragments.get_mut(&CardFragmentKind::Tappable)
    {
        if let Some(tf) = f.as_any_mut().downcast_mut::<TappableFragment>()
        {
            tf.tapped = value;
        }
    }
}
