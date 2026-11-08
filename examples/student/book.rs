//! The curriculum (docs/171): Murphy's *English Grammar in Use* units as a
//! concept hierarchy, the teacher's extra uses, the person's lines, and the
//! held-out sentences nothing is ever told.

/// The curriculum: concept path and example sentences (Murphy, Units 1-7).
pub const UNITS: &[(&str, &[&str])] = &[
    ("grammar/present-continuous", &[
        "she is driving to work", "i am trying to work", "he is having a shower",
        "it is not raining any more", "are you enjoying it",
        "they are building their own house", "the population is increasing very fast",
    ]),
    ("grammar/present-simple", &[
        "nurses look after patients", "the earth goes round the sun",
        "the cafe opens at seven thirty", "rice does not grow in cold climates",
        "i get up at eight every morning", "julie does not drink tea very often",
    ]),
    ("grammar/past-simple", &[
        "i did a lot of work yesterday", "we went to the cinema last night",
        "she worked in a bank for five years", "it did not rain last week",
    ]),
    ("grammar/present-perfect", &[
        "i have done a lot of work today", "she has gone to the bank",
        "we have known each other for years", "it has not rained for a month",
    ]),
];

/// The person: a few of their lines, in the order they said them.
pub const PERSON: &[&str] = &[
    "i think the system should learn on the fly",
    "i think we should keep it simple",
    "the system learns a little and keeps learning",
    "i think meaning is a hierarchy not a word",
    "we should keep the operators simple",
];


/// The teacher's answers to the student's quest (docs/130): more sentences
/// of the same four grammars, shown when the student's knowledge is broken
/// — never the unit names, only the uses.
pub const SHOWN: &[(usize, &str)] = &[
    (0, "the water is boiling can you turn it off"), (0, "my brother is looking for a job"),
    (0, "we are staying at a very nice hotel"), (0, "she is not working this week"),
    (0, "i am getting hungry"), (0, "they are not watching the film"),
    (1, "water boils at a hundred degrees"), (1, "banks lend money to people"),
    (1, "i do not drink coffee at night"), (1, "the museum closes at five"),
    (1, "she speaks four languages"), (1, "we do not watch television very often"),
    (2, "we did not go out last night"), (2, "they arrived at the station an hour ago"),
    (2, "i saw her in town yesterday"), (2, "he did not play very well"),
    (3, "they have lived here for ten years"), (3, "i have lost my keys"),
    (3, "she has not been to the cinema for ages"), (3, "we have just finished dinner"),
];

/// Held out: never told, of the same four grammars, for the one question
/// docs/170 fixed for Q2 — does the concept path beat the flat window?
pub const HELDOUT: &[&str] = &[
    "it is not working very well",
    "she is not staying for dinner",
    "they are not going out tonight",
    "he does not drink tea",
    "it does not rain in the summer",
    "the shop does not open on sunday",
    "we did not watch the film",
    "she did not go to work yesterday",
    "it did not open at five",
    "we have not finished the work",
    "she has not lost her keys",
    "they have not been to the station",
];
