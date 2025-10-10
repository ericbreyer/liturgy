use types::{DayDescription, TrivialDayRank};

struct Breviary {
    vespers: Vespers,
}

struct Vespers {
    antiphons: Location,
    psalms: Location,
    chapter: Location,
    hymn: Location,
    verse: Location,
    magnificat_antiphon: Location,
    collect: Location,
}

enum Location {
    Feria,
    Common,
    Proper,
}

impl Vespers {
    fn from_day_description(day: &DayDescription<TrivialDayRank>) -> Self {
        // Placeholder implementation
        Vespers {
            antiphons: Location::Feria,
            psalms: Location::Feria,
            chapter: Location::Feria,
            hymn: Location::Feria,
            verse: Location::Feria,
            magnificat_antiphon: Location::Feria,
            collect: Location::Feria,
        }
    }
}
