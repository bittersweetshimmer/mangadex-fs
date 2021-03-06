#[derive(Debug, PartialEq, Clone, Copy, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum Genre {
    FourKoma = 1,
    Action = 2,
    Adventure = 3,
    AwardWinning = 4,
    Comedy = 5,
    Cooking = 6,
    Doujinshi = 7,
    Drama = 8,
    Ecchi = 9,
    Fantasy = 10,
    Gyaru = 11,
    Harem = 12,
    Historical = 13,
    Horror = 14,
    MartialArts = 16,
    Mecha = 17,
    Medical = 18,
    Music = 19,
    Mystery = 20,
    Oneshot = 21,
    Psychological = 22,
    Romance = 23,
    SchoolLife = 24,
    SciFi = 25,
    ShoujoAi = 28,
    ShounenAi = 30,
    SliceOfLife = 31,
    Smut = 32,
    Sports = 33,
    Supernatural = 34,
    Tragedy = 35,
    LongStrip = 36,
    Yaoi = 37,
    Yuri = 38,
    VideoGames = 40,
    Isekai = 41,
    Adaptation = 42,
    Anthology = 43,
    WebComic = 44,
    FullColor = 45,
    UserCreated = 46,
    OfficialColored = 47,
    FanColored = 48,
    Gore = 49,
    SexualViolence = 50,
    Crime = 51,
    MagicalGirls = 52,
    Philosophical = 53,
    Superhero = 54,
    Thriller = 55,
    Wuxia = 56,
    Aliens = 57,
    Animals = 58,
    Crossdressing = 59,
    Demons = 60,
    Delinquents = 61,
    Genderswap = 62,
    Ghosts = 63,
    MonsterGirls = 64,
    Loli = 65,
    Magic = 66,
    Military = 67,
    Monsters = 68,
    Ninja = 69,
    OfficeWorkers = 70,
    Police = 71,
    PostApocalyptic = 72,
    Reincarnation = 73,
    ReverseHarem = 74,
    Samurai = 75,
    Shota = 76,
    Survival = 77,
    TimeTravel = 78,
    Vampires = 79,
    TraditionalGames = 80,
    VirtualReality = 81,
    Zombies = 82,
    Incest = 83,
    Mafia = 84
}

impl std::convert::TryFrom<u8> for Genre {
    type Error = ();
    
    fn try_from(id: u8) -> Result<Self, Self::Error> {
        match id {
            1u8 => Ok(Genre::FourKoma),
            2u8 => Ok(Genre::Action),
            3u8 => Ok(Genre::Adventure),
            4u8 => Ok(Genre::AwardWinning),
            5u8 => Ok(Genre::Comedy),
            6u8 => Ok(Genre::Cooking),
            7u8 => Ok(Genre::Doujinshi),
            8u8 => Ok(Genre::Drama),
            9u8 => Ok(Genre::Ecchi),
            10u8 => Ok(Genre::Fantasy),
            11u8 => Ok(Genre::Gyaru),
            12u8 => Ok(Genre::Harem),
            13u8 => Ok(Genre::Historical),
            14u8 => Ok(Genre::Horror),
            16u8 => Ok(Genre::MartialArts),
            17u8 => Ok(Genre::Mecha),
            18u8 => Ok(Genre::Medical),
            19u8 => Ok(Genre::Music),
            20u8 => Ok(Genre::Mystery),
            21u8 => Ok(Genre::Oneshot),
            22u8 => Ok(Genre::Psychological),
            23u8 => Ok(Genre::Romance),
            24u8 => Ok(Genre::SchoolLife),
            25u8 => Ok(Genre::SciFi),
            28u8 => Ok(Genre::ShoujoAi),
            30u8 => Ok(Genre::ShounenAi),
            31u8 => Ok(Genre::SliceOfLife),
            32u8 => Ok(Genre::Smut),
            33u8 => Ok(Genre::Sports),
            34u8 => Ok(Genre::Supernatural),
            35u8 => Ok(Genre::Tragedy),
            36u8 => Ok(Genre::LongStrip),
            37u8 => Ok(Genre::Yaoi),
            38u8 => Ok(Genre::Yuri),
            40u8 => Ok(Genre::VideoGames),
            41u8 => Ok(Genre::Isekai),
            42u8 => Ok(Genre::Adaptation),
            43u8 => Ok(Genre::Anthology),
            44u8 => Ok(Genre::WebComic),
            45u8 => Ok(Genre::FullColor),
            46u8 => Ok(Genre::UserCreated),
            47u8 => Ok(Genre::OfficialColored),
            48u8 => Ok(Genre::FanColored),
            49u8 => Ok(Genre::Gore),
            50u8 => Ok(Genre::SexualViolence),
            51u8 => Ok(Genre::Crime),
            52u8 => Ok(Genre::MagicalGirls),
            53u8 => Ok(Genre::Philosophical),
            54u8 => Ok(Genre::Superhero),
            55u8 => Ok(Genre::Thriller),
            56u8 => Ok(Genre::Wuxia),
            57u8 => Ok(Genre::Aliens),
            58u8 => Ok(Genre::Animals),
            59u8 => Ok(Genre::Crossdressing),
            60u8 => Ok(Genre::Demons),
            61u8 => Ok(Genre::Delinquents),
            62u8 => Ok(Genre::Genderswap),
            63u8 => Ok(Genre::Ghosts),
            64u8 => Ok(Genre::MonsterGirls),
            65u8 => Ok(Genre::Loli),
            66u8 => Ok(Genre::Magic),
            67u8 => Ok(Genre::Military),
            68u8 => Ok(Genre::Monsters),
            69u8 => Ok(Genre::Ninja),
            70u8 => Ok(Genre::OfficeWorkers),
            71u8 => Ok(Genre::Police),
            72u8 => Ok(Genre::PostApocalyptic),
            73u8 => Ok(Genre::Reincarnation),
            74u8 => Ok(Genre::ReverseHarem),
            75u8 => Ok(Genre::Samurai),
            76u8 => Ok(Genre::Shota),
            77u8 => Ok(Genre::Survival),
            78u8 => Ok(Genre::TimeTravel),
            79u8 => Ok(Genre::Vampires),
            80u8 => Ok(Genre::TraditionalGames),
            81u8 => Ok(Genre::VirtualReality),
            82u8 => Ok(Genre::Zombies),
            83u8 => Ok(Genre::Incest),
            84u8 => Ok(Genre::Mafia),
            _ => Err(())
        }
    }
}

impl std::convert::TryFrom<&str> for Genre {
    type Error = ();
    
    fn try_from(id: &str) -> Result<Self, Self::Error> {
        match id {
            "4koma" => Ok(Genre::FourKoma),
            "action" => Ok(Genre::Action),
            "adventure" => Ok(Genre::Adventure),
            "awardwinning" => Ok(Genre::AwardWinning),
            "comedy" => Ok(Genre::Comedy),
            "cooking" => Ok(Genre::Cooking),
            "doujinshi" => Ok(Genre::Doujinshi),
            "drama" => Ok(Genre::Drama),
            "ecchi" => Ok(Genre::Ecchi),
            "fantasy" => Ok(Genre::Fantasy),
            "gyaru" => Ok(Genre::Gyaru),
            "harem" => Ok(Genre::Harem),
            "historical" => Ok(Genre::Historical),
            "horror" => Ok(Genre::Horror),
            "martialarts" => Ok(Genre::MartialArts),
            "mecha" => Ok(Genre::Mecha),
            "medical" => Ok(Genre::Medical),
            "music" => Ok(Genre::Music),
            "mystery" => Ok(Genre::Mystery),
            "oneshot" => Ok(Genre::Oneshot),
            "psychological" => Ok(Genre::Psychological),
            "romance" => Ok(Genre::Romance),
            "schoollife" => Ok(Genre::SchoolLife),
            "scifi" => Ok(Genre::SciFi),
            "shoujoai" => Ok(Genre::ShoujoAi),
            "shounenai" => Ok(Genre::ShounenAi),
            "sliceoflife" => Ok(Genre::SliceOfLife),
            "smut" => Ok(Genre::Smut),
            "sports" => Ok(Genre::Sports),
            "supernatural" => Ok(Genre::Supernatural),
            "tragedy" => Ok(Genre::Tragedy),
            "longstrip" => Ok(Genre::LongStrip),
            "yaoi" => Ok(Genre::Yaoi),
            "yuri" => Ok(Genre::Yuri),
            "videogames" => Ok(Genre::VideoGames),
            "isekai" => Ok(Genre::Isekai),
            "adaptation" => Ok(Genre::Adaptation),
            "anthology" => Ok(Genre::Anthology),
            "webcomic" => Ok(Genre::WebComic),
            "fullcolor" => Ok(Genre::FullColor),
            "usercreated" => Ok(Genre::UserCreated),
            "officialcolored" => Ok(Genre::OfficialColored),
            "fancolored" => Ok(Genre::FanColored),
            "gore" => Ok(Genre::Gore),
            "sexualviolence" => Ok(Genre::SexualViolence),
            "crime" => Ok(Genre::Crime),
            "magicalgirls" => Ok(Genre::MagicalGirls),
            "philosophical" => Ok(Genre::Philosophical),
            "superhero" => Ok(Genre::Superhero),
            "thriller" => Ok(Genre::Thriller),
            "wuxia" => Ok(Genre::Wuxia),
            "aliens" => Ok(Genre::Aliens),
            "animals" => Ok(Genre::Animals),
            "crossdressing" => Ok(Genre::Crossdressing),
            "demons" => Ok(Genre::Demons),
            "delinquents" => Ok(Genre::Delinquents),
            "genderswap" => Ok(Genre::Genderswap),
            "ghosts" => Ok(Genre::Ghosts),
            "monstergirls" => Ok(Genre::MonsterGirls),
            "loli" => Ok(Genre::Loli),
            "magic" => Ok(Genre::Magic),
            "military" => Ok(Genre::Military),
            "monsters" => Ok(Genre::Monsters),
            "ninja" => Ok(Genre::Ninja),
            "officeworkers" => Ok(Genre::OfficeWorkers),
            "police" => Ok(Genre::Police),
            "postapocalyptic" => Ok(Genre::PostApocalyptic),
            "reincarnation" => Ok(Genre::Reincarnation),
            "reverseharem" => Ok(Genre::ReverseHarem),
            "samurai" => Ok(Genre::Samurai),
            "shota" => Ok(Genre::Shota),
            "survival" => Ok(Genre::Survival),
            "timetravel" => Ok(Genre::TimeTravel),
            "vampires" => Ok(Genre::Vampires),
            "traditionalgames" => Ok(Genre::TraditionalGames),
            "virtualreality" => Ok(Genre::VirtualReality),
            "zombies" => Ok(Genre::Zombies),
            "incest" => Ok(Genre::Incest),
            "mafia" => Ok(Genre::Mafia),
            _ => Err(())
        }
    }
}