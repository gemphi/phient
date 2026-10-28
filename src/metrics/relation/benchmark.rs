#[allow(unused_imports)]
use super::*;

mod rank;
mod family;

impl RelationBenchmark {
    /// The default probe set: 305 pairs across 10 relation families.
    ///
    /// The previous set was 23 usable pairs across three families, which is too
    /// few to support any claim it was being used to make: an analogy@1 of
    /// 10.49% on 23 pairs is a handful of hits, and a single family could carry
    /// the total on its own. Task A2 grew it to the point where an effect has to
    /// clear its own error bar.
    ///
    /// The families are chosen so that a failure is the model's and not the
    /// probe's: each relation is unambiguous, and every word is one a
    /// nineteenth-century dictionary defines. They also span the two kinds of
    /// relation separately - **semantic** (gender, antonym, hypernym,
    /// nationality) and **morphological** (number, comparative, past tense,
    /// agent, quality, negation) - because a manifold can easily learn the
    /// second from spelling alone while learning nothing about meaning, and a
    /// combined score would hide that. Read the per-family breakdown, not the
    /// total.
    ///
    /// Families whose vocabulary a given corpus does not cover report zero
    /// usable pairs rather than silently contributing nothing, which is why
    /// `usable_pairs` is on [`FamilyResult`].
    pub fn default_families() -> Vec<RelationFamily> {
        let fam = |name: &str, pairs: &[(&str, &str)]| RelationFamily {
            name: name.to_string(),
            pairs: pairs
                .iter()
                .map(|(a, b)| RelationPair { a: a.to_string(), b: b.to_string() })
                .collect(),
        };

        vec![
            fam(
                "gender",
                &[
                    ("man", "woman"), ("boy", "girl"), ("king", "queen"),
                    ("father", "mother"), ("son", "daughter"), ("brother", "sister"),
                    ("uncle", "aunt"), ("nephew", "niece"), ("husband", "wife"),
                    ("grandfather", "grandmother"), ("sir", "madam"), ("lord", "lady"),
                    ("prince", "princess"), ("actor", "actress"), ("waiter", "waitress"),
                    ("host", "hostess"), ("widower", "widow"), ("bull", "cow"),
                    ("rooster", "hen"), ("stallion", "mare"), ("ram", "ewe"),
                    ("buck", "doe"), ("drake", "duck"), ("gander", "goose"),
                    ("lion", "lioness"), ("tiger", "tigress"), ("emperor", "empress"),
                    ("duke", "duchess"), ("master", "mistress"), ("monk", "nun"),
                ],
            ),
            fam(
                "number",
                &[
                    ("man", "men"), ("woman", "women"), ("child", "children"),
                    ("foot", "feet"), ("tooth", "teeth"), ("mouse", "mice"),
                    ("goose", "geese"), ("ox", "oxen"), ("person", "people"),
                    ("leaf", "leaves"), ("knife", "knives"), ("wife", "wives"),
                    ("life", "lives"), ("half", "halves"), ("loaf", "loaves"),
                    ("thief", "thieves"), ("shelf", "shelves"), ("wolf", "wolves"),
                    ("calf", "calves"), ("city", "cities"), ("baby", "babies"),
                    ("lady", "ladies"), ("story", "stories"), ("country", "countries"),
                    ("family", "families"), ("army", "armies"), ("party", "parties"),
                    ("body", "bodies"), ("hero", "heroes"), ("potato", "potatoes"),
                    ("echo", "echoes"), ("box", "boxes"), ("church", "churches"),
                    ("brush", "brushes"), ("glass", "glasses"),
                ],
            ),
            fam(
                "antonym",
                &[
                    ("hot", "cold"), ("big", "small"), ("light", "dark"),
                    ("high", "low"), ("fast", "slow"), ("hard", "soft"),
                    ("good", "bad"), ("long", "short"), ("wide", "narrow"),
                    ("thick", "thin"), ("strong", "weak"), ("rich", "poor"),
                    ("young", "old"), ("happy", "sad"), ("wet", "dry"),
                    ("clean", "dirty"), ("full", "empty"), ("deep", "shallow"),
                    ("sharp", "blunt"), ("sweet", "bitter"), ("smooth", "rough"),
                    ("loud", "quiet"), ("brave", "cowardly"), ("wise", "foolish"),
                    ("true", "false"), ("right", "wrong"), ("love", "hate"),
                    ("war", "peace"), ("birth", "death"), ("day", "night"),
                    ("summer", "winter"), ("north", "south"), ("east", "west"),
                    ("up", "down"), ("joy", "sorrow"),
                ],
            ),
            fam(
                "comparative",
                &[
                    ("big", "bigger"), ("small", "smaller"), ("long", "longer"),
                    ("short", "shorter"), ("high", "higher"), ("low", "lower"),
                    ("fast", "faster"), ("slow", "slower"), ("hard", "harder"),
                    ("soft", "softer"), ("strong", "stronger"), ("weak", "weaker"),
                    ("rich", "richer"), ("poor", "poorer"), ("young", "younger"),
                    ("old", "older"), ("warm", "warmer"), ("cold", "colder"),
                    ("deep", "deeper"), ("wide", "wider"), ("narrow", "narrower"),
                    ("thick", "thicker"), ("thin", "thinner"), ("sweet", "sweeter"),
                    ("bright", "brighter"), ("dark", "darker"), ("clear", "clearer"),
                    ("sharp", "sharper"), ("smooth", "smoother"), ("quick", "quicker"),
                ],
            ),
            fam(
                "past_tense",
                &[
                    ("walk", "walked"), ("talk", "talked"), ("work", "worked"),
                    ("play", "played"), ("look", "looked"), ("call", "called"),
                    ("want", "wanted"), ("need", "needed"), ("help", "helped"),
                    ("open", "opened"), ("close", "closed"), ("watch", "watched"),
                    ("learn", "learned"), ("turn", "turned"), ("start", "started"),
                    ("follow", "followed"), ("live", "lived"), ("move", "moved"),
                    ("love", "loved"), ("use", "used"), ("ask", "asked"),
                    ("answer", "answered"), ("carry", "carried"), ("marry", "married"),
                    ("study", "studied"), ("try", "tried"), ("cry", "cried"),
                    ("stop", "stopped"), ("drop", "dropped"), ("go", "went"),
                    ("see", "saw"), ("take", "took"), ("give", "gave"),
                    ("come", "came"), ("know", "knew"),
                ],
            ),
            fam(
                "agent",
                &[
                    ("teach", "teacher"), ("write", "writer"), ("sing", "singer"),
                    ("dance", "dancer"), ("paint", "painter"), ("farm", "farmer"),
                    ("work", "worker"), ("play", "player"), ("read", "reader"),
                    ("speak", "speaker"), ("lead", "leader"), ("build", "builder"),
                    ("bake", "baker"), ("hunt", "hunter"), ("drive", "driver"),
                    ("ride", "rider"), ("run", "runner"), ("swim", "swimmer"),
                    ("fight", "fighter"), ("print", "printer"), ("own", "owner"),
                    ("buy", "buyer"), ("sell", "seller"), ("help", "helper"),
                    ("rule", "ruler"), ("found", "founder"), ("keep", "keeper"),
                    ("make", "maker"), ("give", "giver"), ("follow", "follower"),
                ],
            ),
            fam(
                "hypernym",
                &[
                    ("dog", "animal"), ("oak", "tree"), ("rose", "flower"),
                    ("sparrow", "bird"), ("salmon", "fish"), ("copper", "metal"),
                    ("granite", "rock"), ("wheat", "grain"), ("apple", "fruit"),
                    ("hammer", "tool"), ("chair", "furniture"), ("violin", "instrument"),
                    ("hydrogen", "element"), ("triangle", "figure"), ("circle", "figure"),
                    ("crimson", "color"), ("azure", "color"), ("monday", "day"),
                    ("january", "month"), ("spring", "season"), ("gold", "metal"),
                    ("silver", "metal"), ("pine", "tree"), ("eagle", "bird"),
                    ("trout", "fish"), ("wine", "drink"), ("bread", "food"),
                    ("cotton", "fiber"), ("marble", "stone"), ("clay", "earth"),
                    ("rifle", "weapon"), ("sword", "weapon"), ("ship", "vessel"),
                    ("wagon", "vehicle"), ("cottage", "building"),
                ],
            ),
            fam(
                "nationality",
                &[
                    ("france", "french"), ("spain", "spanish"), ("england", "english"),
                    ("germany", "german"), ("italy", "italian"), ("russia", "russian"),
                    ("china", "chinese"), ("japan", "japanese"), ("greece", "greek"),
                    ("egypt", "egyptian"), ("ireland", "irish"), ("scotland", "scottish"),
                    ("poland", "polish"), ("sweden", "swedish"), ("denmark", "danish"),
                    ("norway", "norwegian"), ("holland", "dutch"), ("portugal", "portuguese"),
                    ("turkey", "turkish"), ("persia", "persian"), ("india", "indian"),
                    ("arabia", "arabian"), ("africa", "african"), ("america", "american"),
                    ("europe", "european"),
                ],
            ),
            fam(
                "quality",
                &[
                    ("kind", "kindness"), ("dark", "darkness"), ("weak", "weakness"),
                    ("good", "goodness"), ("sad", "sadness"), ("happy", "happiness"),
                    ("bitter", "bitterness"), ("sweet", "sweetness"), ("bold", "boldness"),
                    ("sick", "sickness"), ("blind", "blindness"), ("thick", "thickness"),
                    ("great", "greatness"), ("hard", "hardness"), ("soft", "softness"),
                    ("mad", "madness"), ("glad", "gladness"), ("quick", "quickness"),
                    ("sharp", "sharpness"), ("smooth", "smoothness"), ("rough", "roughness"),
                    ("bright", "brightness"), ("still", "stillness"), ("ready", "readiness"),
                    ("holy", "holiness"),
                ],
            ),
            fam(
                "negation",
                &[
                    ("happy", "unhappy"), ("kind", "unkind"), ("known", "unknown"),
                    ("able", "unable"), ("just", "unjust"), ("equal", "unequal"),
                    ("common", "uncommon"), ("certain", "uncertain"), ("easy", "uneasy"),
                    ("fair", "unfair"), ("fit", "unfit"), ("holy", "unholy"),
                    ("lucky", "unlucky"), ("natural", "unnatural"), ("pleasant", "unpleasant"),
                    ("ripe", "unripe"), ("safe", "unsafe"), ("seen", "unseen"),
                    ("true", "untrue"), ("usual", "unusual"), ("wise", "unwise"),
                    ("worthy", "unworthy"), ("clean", "unclean"), ("done", "undone"),
                    ("even", "uneven"),
                ],
            ),
        ]
    }
}
