//!
//! ## String Literals
//!

use avjason_macros::{verbatim as v, Spanned, SpecRef};

use crate::{
    common::{Source, Span},
    lexing::{LexError, LexResult, LexT, Many, SourceStream},
};

use super::{
    escapes::EscapeSequence,
    line_terminator::{is_line_terminator, LineTerminatorSequence},
};

///
/// String literals.
///
#[SpecRef("JSON5String")]
#[derive(Debug, Spanned)]
pub enum LString {
    Double(v!('"'), Many<StringPart<"\"">>, v!('"')),
    Single(v!('\''), Many<StringPart<"'">>, v!('\'')),
}

///
/// All possible parts of a string literal.
///
#[derive(Debug, Spanned)]
pub enum StringPart<const D: &'static str> {
    Char(StringChar<D>),
    Escape(v!('\\'), EscapeSequence),
    LineContinuation(v!('\\'), LineTerminatorSequence),
    LS(v!('\u{2028}')),
    PS(v!('\u{2029}')),
}

///
/// A non-escaped string character.
///
/// This represents itself.
///
#[derive(Debug, Spanned)]
pub struct StringChar<const D: &'static str> {
    span: Span,
    raw: char,
}

// ---

impl LexT for LString {
    fn peek<S: Source>(input: &SourceStream<S>) -> bool {
        <v!('"') as LexT>::peek(input) || <v!('\'') as LexT>::peek(input)
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> Result<Self, LexError> {
        input
            .lex()
            .and(|opening| {
                let contents = input.lex()?;
                let closing = input.lex().expected_msg(input, "Expected closing `\"`")?;
                LexResult::Lexed(Self::Double(opening, contents, closing))
            })
            .or(|| {
                input.lex().and(|opening| {
                    let contents = input.lex()?;
                    let closing = input.lex().expected_msg(input, "Expected closing `'`")?;
                    LexResult::Lexed(Self::Single(opening, contents, closing))
                })
            })
            .unwrap_as_result()
    }
}

impl<const D: &'static str> LexT for StringPart<D> {
    fn peek<S: Source>(input: &SourceStream<S>) -> bool {
        <StringChar<D> as LexT>::peek(input)
            || <v!('\\') as LexT>::peek(input)
            || <v!('\u{2028}') as LexT>::peek(input)
            || <v!('\u{2029}') as LexT>::peek(input)
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> Result<Self, LexError> {
        // Some real nastiness going on here:
        //      essentially, complex functional-like control flow
        //      for the \ character to check if either .

        // .unwrap_as_result() ok since Self::peek()
        input
            .lex()
            .map(Self::LS)
            .or(|| input.lex().map(Self::PS))
            .or(|| input.lex().map(Self::Char))
            .or(|| {
                input.lex().and(|backslash: v!('\\')| {
                    input
                        .lex()
                        .map(|esc| Self::Escape(backslash.clone(), esc))
                        .or(|| {
                            LexResult::Lexed(Self::LineContinuation(
                                backslash,
                                input.lex().expected_msg(
                                    input,
                                    "Expected either an escape code here, or newline; got neither.",
                                )?,
                            ))
                        })
                })
            })
            .unwrap_as_result()
    }
}

impl<const D: &'static str> LexT for StringChar<D> {
    fn peek<S: Source>(input: &SourceStream<S>) -> bool {
        !(input.upcoming(D) || input.upcoming(is_line_terminator) || input.upcoming("\\"))
            && input.peek().is_some()
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> Result<Self, LexError> {
        // .unwrap() ok since Self::peek() -> next character exists.
        let (loc, raw) = input.take().unwrap();

        Ok(Self {
            span: Span::from(loc),
            raw,
        })
    }
}

// ---

///
/// The character value of a part of a string literal, which
/// dictates which character that part represents.
///
/// See the [ECMAScript spec](https://262.ecma-international.org/5.1/#sec-7.8.4).
///
pub trait CharacterValue {
    ///
    /// Encodes the utf-16 based character value into a
    /// buffer, returning a slice of the bytes used.
    ///
    fn cv<'a, 'b: 'a>(&'a self, buf: &'b mut [u16; 2]) -> &'b [u16];

    ///
    /// Attempts to convert this utf-16 as a Rust char.
    ///
    fn try_as_char(&self) -> Option<char> {
        let buf = &mut [0u16; 2];

        let mut a = char::decode_utf16(self.cv(buf).iter().copied());
        a.next().and_then(Result::ok)
    }
}

///
/// The value a string literal represents.
///
/// See the [ECMAScript spec](https://262.ecma-international.org/5.1/#sec-7.8.4).
///
pub trait StringValue {
    ///
    /// Because this is ECMAScript, strings are utf-16 encoded
    /// &mdash; this will be preserved at this stage.
    ///
    fn sv(&self) -> Vec<u16>;

    ///
    /// Workaround for testing only.
    ///
    #[cfg(test)]
    fn to_rust_string_lossy(&self) -> String {
        let utf16 = self.sv();
        String::from_utf16_lossy(&utf16)
    }
}

// ---

impl<const D: &'static str> CharacterValue for StringPart<D> {
    fn cv<'a, 'b: 'a>(&'a self, buf: &'b mut [u16; 2]) -> &'b [u16] {
        match self {
            StringPart::Char(ch) => ch.cv(buf),
            StringPart::Escape(_, esc) => esc.cv(buf),
            StringPart::LineContinuation(_, _) => &buf[0..0], // Skip.
            StringPart::LS(_) => '\u{2028}'.encode_utf16(buf),
            StringPart::PS(_) => '\u{2029}'.encode_utf16(buf),
        }
    }
}

impl<const D: &'static str> CharacterValue for StringChar<D> {
    fn cv<'a, 'b: 'a>(&'a self, buf: &'b mut [u16; 2]) -> &'b [u16] {
        self.raw.encode_utf16(buf)
    }
}

// ---

impl StringValue for LString {
    fn sv(&self) -> Vec<u16> {
        match self {
            LString::Double(_, contents, _) => contents.sv(),
            LString::Single(_, contents, _) => contents.sv(),
        }
    }
}

///
/// Collect character values as a UTF-16 string.
///
pub fn collect_cv_into_utf16<'a, CV: CharacterValue + 'a>(
    iter: impl IntoIterator<Item = &'a CV> + 'a,
) -> Vec<u16> {
    let iter: Vec<_> = iter.into_iter().collect();
    // Complete guesswork about the initial capacity:
    // I'm assuming that we're not going to get too many multi-u16 chars.
    let mut string = Vec::with_capacity(iter.len() * 5 / 4);

    let buf = &mut [0; 2];
    for part in iter {
        string.extend(part.cv(buf))
    }

    string
}

impl<const D: &'static str> StringValue for Many<StringPart<D>> {
    fn sv(&self) -> Vec<u16> {
        collect_cv_into_utf16(self.iter())
    }
}
// ---

#[cfg(test)]
mod tests {
    use crate::{
        common::{file::SourceFile, Source},
        lexing::{tokens::string::StringValue, LexResult},
    };

    use super::LString;

    fn test_string(st: &'static str) -> LexResult<LString> {
        let source = SourceFile::dummy_file(st);
        let input = &mut source.stream();
        input.lex()
    }

    #[test]
    fn normal_use_case() {
        assert_eq!(
            test_string(r"'AvdanOS is a community-led open-source project that attempts to implement Avdan\'s \'AvdanOS\' concept as a Wayland compositor.'")
                .unwrap().to_rust_string_lossy(),
            "AvdanOS is a community-led open-source project that attempts to implement Avdan\'s \'AvdanOS\' concept as a Wayland compositor."
        );
    }

    #[test]
    fn empty_string() {
        assert_eq!(test_string("''").unwrap().to_rust_string_lossy(), "");
        assert_eq!(test_string("\"\"").unwrap().to_rust_string_lossy(), "");
    }

    #[test]
    fn escapes() {
        let lit = test_string(
            r"'\'\\\b\f\n\r\t\v\a\!\£\%\*\&\-\=\💩\0\x20\x26\x25\x3c\u0000\u2AFC\u6798\u1623'",
        )
        .expect("Valid parse");

        assert_eq!(
            lit.sv(),
            // Answer from JavaScript (Chrome's V8).
            vec![
                39, 92, 8, 12, 10, 13, 9, 11, 97, 33, 163, 37, 42, 38, 45, 61, 55357, 56489, 0, 32,
                38, 37, 60, 0, 11004, 26520, 5667
            ]
        )
    }

    #[test]
    fn unbalanced_quotes() {
        test_string(r"'Think this is unbalanced -- have you seen capitalism?").unwrap_err();
        test_string(r"'They don\'t let dogs in prison, Grommit! They\'ll put you down!\'")
            .unwrap_err();
        test_string("\"Nothing is more appealing right now than a cup of coffee").unwrap_err();
        test_string("\"Have you heard about the album 'Nervermind'?\\\"").unwrap_err();
    }

    #[test]
    fn invalid_escapes() {
        test_string(r"'\x2'").unwrap_err();
        test_string(r"'\xSS'").unwrap_err();
        test_string(r"'\uSFAA'").unwrap_err();
        test_string(r"'\u2AA'").unwrap_err();

        // It turns out that this form of escape is, in fact, octal.
        // This is not mentioned in the official ECMAScript spec,
        // But is in the optional extenstions: Section B.1.2(https://262.ecma-international.org/5.1/#sec-B.1.2).

        // For example, Node (V8) supports this, but Bun (JavaScriptCore) does not.
        // As it is not mentioned whether to comply with optional extensions,
        // this crate will not be implementing octal syntax.
        test_string(r"'\1'").unwrap_err();
    }

    ///
    /// Random series of u16's interpreted as
    /// string literals, with the utf-16 value
    /// compared to V8's answer.
    ///
    #[test]
    #[allow(text_direction_codepoint_in_literal)]
    fn fuzzing() {
        assert_eq!(
            test_string(r"'䂞ᤴ쭜ؚ洲綏뤒힓蔫黮뱏꽻ꜵ킩악\x19젏◣愜ꏟ醟㾊䑥뷜筵읩ꡓむ髇阏⍉딴퓼됪璮轫ʢ톽觻䀫ꮳ玐耠綈亄宅坍♳ꯑ\uDBCD㇀甚渭￐㛓魴矮︄跕鹞㉋᪽ꎓ鸩먾汕䱏쏀㘓씩㕟챬ᆀ瓅㫱భd瀒峊ﾂꮫ뀥靺㉏ꙓⷳᨾ짽ꑙΥפ肜혃ᐜ恴婁⛫╴䰛⾁\x9A䚠댂䜡ૢ￤ꊠ⧽랸儔根햩쫹輤Ȫ䜭ﺆᬒ偠⊽Ṑ敇봅¨팔檵\uDBB9Գ౓ถ啼摚㿓껠͛躏湜㵬褤쵐㽴䒦迼\uD933ᛳ뵁楻뤣璻㰒\uDB11疲ᆐ腻抐즲ଉ灮鷋䝡밶ꛃ\uDF4Br⯝ଆ㷍工좭澏挣\uDC83◘语开劊椢䀐럵갿懼嗵⊫ꑬ縭郁얱仁༅ⷬ垉₍荌ﵙ䭿⦤牐詌撸উ崙\uDE8E荓畨ꯔᇤ垯蠐⏧쨁▏賈⇜欁ꓕ⍎讷∥㫲画鴶醎迚崴쭹짲교뎈噍⽚\uDFB8냅㥤射'")
                .expect("Valid parse").sv(),
            vec![
                16542, 6452, 52060, 1562, 27954, 32143, 47378, 55187, 34091, 40686, 48207, 44923, 42805, 53417, 50501, 25, 51215, 9699, 62034, 24860, 41951, 37279, 16266, 57468, 17509, 57866, 48604, 31605, 51049, 43091, 12416, 39623, 38415, 9033, 58372, 46388, 54524, 46122, 29870, 36715, 674, 53693, 35323, 16427, 43955, 29584, 32800, 32136, 20100, 64004, 60979, 22349, 9843, 58280, 43985, 56269, 12736, 29978, 28205, 65488, 14035, 39796, 60797, 30702, 65028, 36309, 40542, 12875, 6845, 41875, 40489, 61197, 47678, 27733, 19535, 50112, 13843, 50473, 13663, 52332, 4480, 29893, 15089, 3117, 100, 28690, 23754, 65410, 43947, 45093, 38778, 12879, 42579, 61708, 11763, 6718, 51709, 42073, 933, 1508, 32924, 54787, 5148, 24692, 23105, 9963, 9588, 19483, 12161, 154, 18080, 45826, 18209, 2786, 65508, 41632, 10749, 47032, 20756, 26681, 54697, 51961, 57412, 36644, 554, 18221, 65158, 6930, 20576, 62494, 8893, 7760, 25927, 48389, 168, 58805, 54036, 27317, 56249, 1331, 3155, 3606, 21884, 25690, 16339, 44768, 859, 36495, 28252, 15724, 35108, 52560, 16244, 61134, 17574, 36860, 55603, 5875, 48449, 27003, 47395, 29883, 15378, 56081, 61909, 30130, 4496, 33147, 61001, 25232, 51634, 2825, 28782, 58861, 40395, 18273, 48182, 42691, 57163, 114, 11229, 2822, 15821, 24037, 60822, 51373, 28559, 25379, 62890, 56451, 9688, 35821, 59961, 24320, 21130, 26914, 16400, 47093, 44095, 25084, 22005, 8875, 42092, 32301, 37057, 50609, 20161, 3845, 11756, 22409, 8333, 33612, 64857, 19327, 10660, 29264, 35404, 25784, 2441, 23833, 58025, 58894, 56974, 33619, 61599, 30056, 43988, 4580, 22447, 34832, 9191, 51713, 9615, 36040, 8668, 27393, 42197, 9038, 35767, 8741, 15090, 64163, 40246, 37262, 36826, 23860, 52089, 51698, 44368, 45960, 22093, 12122, 57272, 45253, 14692, 23556
            ]
        );

        assert_eq!(
            test_string(r"'秚놰ꚋ⾏＜給齌걿괔鍺江ﬧ䭑钣ﾬ茊琳株໶杴칽\uDCB1渾⭮ⶕ墢啐渍홦䳹艘紕혺镨쾋冻喢喚䣳㙤봽级邒ថ\uD9B8ោ䋀껄䦐椴⎨譴꽲沺᷆롥ᗐ赙쿰⸲᪘꿲鏸帠梯튋궳　㌦땭ӂ咶鞝卓硄뷬䫾ୢ蘪ク㉃᯲ຮ೚⃊̽詁ꓔ㴺뮢׳Թ尀塠鶈퟾뷊娈鶍х㍣铽렑轨ߵꧮ㒉콳$ꖃ붟섈⟃ᰫ턖\uDAABＬꄅ\uDEE2鰔程륡㩜旎ᢛ᜴휫澜䬁쾘྾퍂畐囃꺴ነ泴얽㤢瀊Ⱃྡྷ뙷輇ዉаￇ㠮㚾졲揿䠭஍磡༛논렺鵠篩㣴셑拨튮ꈌἛ隸눙埊㙺겓셀꠱♌\uDD7E䂼귘檚홗誚͔ꦣ锴ߓ\uDB03匷䏄膟鿕僥粡塕ꎟ宗彲댙䈹⟚ད軵픣㇅燺盰籞睻䋫얨♶኶車\uDBD1젔䖬⬓࣌㺓ྂ꤯⽊᫖ᚋ焹￲甃ꇢ뛉芀ฑ訾蔾\uD96C捈㮙཯㜄'")
                .expect("Valid parse").sv(),
            vec![31194, 45488, 62680, 42635, 12175, 65308, 32102, 40780, 44159, 44308, 37754, 27743, 64295, 19281, 38051, 65452, 33546, 29747, 26666, 3830, 26484, 52861, 56497, 28222, 11118, 11669, 22690, 21840, 28173, 54886, 19705, 58159, 33368, 32021, 54842, 59331, 62622, 38248, 53131, 20923, 21922, 21914, 18675, 13924, 48445, 61070, 32423, 37010, 6032, 55736, 6084, 17088, 44740, 18832, 26932, 9128, 35700, 61865, 44914, 27834, 7622, 47205, 5584, 36185, 53232, 11826, 6808, 45042, 37880, 24096, 26799, 53899, 44467, 12288, 13094, 46445, 1218, 21686, 38813, 21331, 30788, 59530, 61378, 58739, 48620, 19198, 2914, 34346, 12463, 12867, 7154, 3758, 3290, 8394, 829, 35393, 42196, 15674, 48034, 1523, 1337, 23552, 22624, 40328, 55294, 48586, 23048, 40333, 63659, 1093, 13155, 38141, 47121, 36712, 2037, 43502, 13449, 53107, 36, 42371, 48543, 49416, 10179, 7211, 53526, 55979, 65324, 41221, 57058, 39956, 31243, 47457, 14940, 26062, 6299, 5940, 55083, 28572, 19201, 53144, 4030, 54082, 30032, 22211, 44724, 4752, 27892, 59533, 50621, 14626, 28682, 11283, 4002, 46711, 36615, 4809, 1072, 65479, 14382, 14014, 51314, 25599, 18477, 2957, 30945, 3867, 45436, 47162, 40288, 31721, 14580, 49489, 25320, 53934, 41484, 7963, 38584, 45593, 22474, 13946, 44179, 57572, 49472, 43057, 63050, 9804, 62475, 56702, 16572, 44504, 27290, 57604, 54871, 35482, 852, 43427, 38196, 2003, 56067, 21303, 17348, 33183, 40917, 20709, 31905, 22613, 41887, 23447, 24434, 45849, 60174, 16953, 10202, 3921, 36597, 60241, 54563, 12741, 29178, 30448, 31838, 30587, 17131, 58558, 50600, 9846, 4790, 63746, 56273, 51220, 17836, 11027, 2252, 16019, 3970, 43311, 12106, 59874, 6870, 5771, 28985, 57731, 65522, 29955, 41442, 57924, 46793, 33408, 3601, 35390, 34110, 55660, 25416, 15257, 3951, 14084
            ]
        );

        assert_eq!(
            test_string(r"'ᐇ➢ᷙ榃훳휆ꅦ欥㒎ᩀஒ䧓㼿\uDBBE䍷ख़ꔬ쳩呍ꑼ᧡譶䮿뽕ꙴ뢪촗㲪袹쟓Ὴ棅捈批쟹砛▟즣㎜펒巵ꚓ꜐Ꞝ톘ᅿ㣓䐩籮晤饳堓䋤੡㇪ᾚ厤秲猪絓ꡨ俛붷継㤕᠍䌖\uDEDA砇₈㴹牙뛞ꃤ˕ඟ蚍醚픦먜ἺṴ茫뚯﹤唟풰섙碁젋졂∽赞摖隆걑쩒柀瞛擧获㺟染ሏ᧻사汋迪셨㸹嵂䤬闄䏇䒘㎓뻑ꭊ圹衁끇ᮓ빪耔怮⺇䳐묃䅻׫磼脉ò᷾姰佄鶕붬ጛ갲祔奔㖔⣪℄蝱靦ꆯꜮ궻씍ﹶ쑿鞾轪⠱胼螓멣栟跦沭⊾夏尃먗㲳瀆ퟎ콆攂喉ㄻ嶳鸹䉭뾐铥䤰漘뉦ᅭ䐨钞薑涐⹾쏾䏔蝶フ⯐ຒ藧㒴緽듹⇒㛎明黩㛳氓梛辽\uD850㣞\uDD33鼚暤梅㧊Ҩᩰ圄찅甦信矆誐ոꖚ冐䞭㹳㹆鰑'")
                .expect("Valid parse").sv(),
            vec![59730, 5127, 10146, 7641, 59098, 27011, 55027, 55046, 41318, 27429, 13454, 6720, 2962, 63467, 18899, 16191, 56254, 17271, 2393, 42284, 52457, 21581, 42108, 6625, 63025, 35702, 59558, 19391, 48981, 42612, 47274, 52503, 15530, 35001, 51155, 8138, 26821, 25416, 25209, 51193, 59246, 30747, 9631, 51619, 13212, 54162, 24053, 42643, 42768, 42908, 53656, 4479, 14547, 17449, 31854, 26212, 39283, 22547, 17124, 2657, 60174, 12778, 8090, 21412, 31218, 29482, 32083, 63500, 43112, 20443, 48567, 32153, 14613, 6157, 17174, 58187, 57050, 30727, 8328, 15673, 29273, 46814, 41188, 725, 3487, 34445, 37274, 54566, 47644, 7994, 7796, 33579, 46767, 65124, 21791, 54448, 49433, 30849, 51211, 51266, 8765, 63507, 36190, 25686, 38534, 44113, 51794, 26560, 30619, 58556, 25831, 33719, 16031, 61982, 26579, 4623, 6651, 57722, 49324, 27723, 36842, 49512, 15929, 23874, 58228, 18732, 38340, 17351, 17560, 13203, 48849, 43850, 61644, 22329, 34881, 45127, 7059, 48746, 32788, 24622, 11911, 19664, 47875, 16763, 1515, 30972, 33033, 242, 7678, 23024, 20292, 40341, 48556, 4891, 44082, 31060, 22868, 13716, 10474, 8452, 34673, 60035, 38758, 41391, 62827, 42798, 44475, 63743, 58786, 50445, 62643, 65142, 50303, 38846, 36714, 10289, 58929, 33020, 34707, 47715, 26655, 36326, 27821, 62815, 8894, 22799, 23555, 47639, 60265, 15539, 28678, 58432, 55246, 53062, 25858, 60646, 21897, 60958, 12603, 23987, 40505, 17005, 49040, 38117, 18736, 28440, 45670, 4461, 17448, 38046, 34193, 28048, 11902, 50174, 17364, 34678, 12501, 11216, 3730, 34279, 13492, 32253, 46329, 8658, 60048, 14030, 26126, 40681, 14067, 27667, 61088, 26779, 36797, 55376, 14558, 56627, 40730, 60610, 26276, 26757, 14794, 1192, 6768, 22276, 52229, 29990, 20449, 30662, 35472, 1400, 42394, 20880, 18349, 15987, 15942, 39953
            ]
        );

        assert_eq!(
            test_string(r"'祐䇛珈䣏둫䠽㩅⏇ᗊꥷ⛙寎杅똦儣桴糎絪㋢雳쑢㡟ⓘ譏笜穘ᎏ난ᡂᣕ䯹嗔楗鏯⼺㌨떟ሎỬ⵹䪋౿⸬ה\uDCAE釉萳阪櫒洈宀駅뻍슘ᴘ錱⎝ᓛ堼䲃㖭鸜鸍\uDABC掰ｶ픸⑘佫䔻樟嗌軓\x83喋瀛䙳峦튬酥㫑䔶␱씿芩鵙䗲衜賈\uDF4B㋋颡쩾敯侥㰟ᱍሇ笿뭦ۑ؄ࡾ갆쌨嬓ꑌ⼮犥䏧擌臤ꋪ꡿↊됰㏞讐റᲱ篤ⴓꚹ菙䪦엇⡗袦嵻嶬捡쭇䙑婫⏱韲흛⌠ꊜ볲緙덕㣔鍸暐䄧뭝鳴ᙇ莯覧⑩쿿벹⦠紈ۃ戎쥔븗ꍏ桝\uDCD8໗坊圻賈꧙볰ꁣ칏ᄩ\uDC7F삃爞겕虵䡏έ홰⸱焸왵⒗㚪좵Ϗ훱熞䗳၇ェ죳ਙ\uDE73\uD8E8汃ᚠ鏮恹⺐죾ﮒ툔퐬ઇﵔ촶⊄෈᬴늮\uDE3F䣓攙蘿儠ූആ䞔ꄶ亏㘝迬'")
                .expect("Valid parse").sv(),
            vec![31056, 57578, 16859, 29640, 18639, 58428, 46187, 18493, 14917, 9159, 5578, 43383, 9945, 23502, 26437, 46630, 20771, 26740, 31950, 32106, 59267, 13026, 38643, 59963, 58248, 50274, 14431, 9432, 35663, 31516, 31320, 5007, 45212, 6210, 6357, 63623, 19449, 60661, 21972, 26967, 37871, 12090, 58152, 13096, 46495, 62016, 4622, 7916, 11641, 19083, 3199, 11820, 1492, 56494, 37321, 33843, 38442, 27346, 27912, 23424, 39365, 48845, 49816, 7448, 37681, 9117, 5339, 22588, 19587, 58498, 13741, 40476, 40461, 55996, 25520, 62609, 65398, 54584, 9304, 20331, 57805, 17723, 27167, 61366, 57712, 57423, 21964, 36563, 131, 21899, 28699, 18035, 23782, 53932, 37221, 61779, 15057, 17718, 9265, 50495, 33449, 40281, 59399, 17906, 59923, 34908, 36040, 61662, 57163, 13003, 39073, 51838, 25967, 20389, 15391, 7245, 60883, 4615, 31551, 47974, 1745, 1540, 2174, 44038, 49960, 58601, 23315, 42060, 12078, 29349, 17383, 25804, 33252, 41706, 43135, 8586, 46128, 13278, 35728, 3377, 57603, 7345, 31716, 11539, 42681, 33753, 19110, 50631, 10327, 34982, 23931, 60456, 23980, 25441, 52039, 63224, 18001, 23147, 9201, 38898, 55131, 8992, 58355, 41628, 48370, 32217, 45909, 14548, 37752, 26256, 16679, 57404, 47965, 40180, 5703, 33711, 35239, 9321, 53247, 48313, 10656, 32008, 1731, 25102, 51540, 48663, 41807, 26717, 56536, 3799, 22346, 22331, 63747, 43481, 48368, 41059, 52815, 58274, 4393, 56447, 49283, 29214, 44181, 34421, 18511, 8051, 62021, 54896, 11825, 28984, 50805, 9367, 13994, 51381, 975, 55025, 57379, 29086, 17907, 4167, 12455, 51443, 2585, 56947, 55528, 27715, 5792, 59015, 37870, 24697, 11920, 51454, 64402, 53780, 54316, 2695, 64852, 52534, 8836, 3528, 6964, 45742, 56895, 18643, 62500, 25881, 63760, 20768, 3542, 3334, 18324, 41270, 20111, 13853, 61106, 36844
            ]
        );

        // This one broke clippy, because text changes directions halfway through,
        // but we don't care about that!
        assert_eq!(
            test_string(r"'䊄㨍䕇㉆鹹䤑謲虉喙帺⫮૚謤㵳骼뜜ᳪﱞ䀅ߢ兾ỷ煡鼱뚹ꕖ䜻\uDC9F終蚔㏼뫨軗쯰붰줓城鱃膫⌶틧ﲔ醛㹣䳵踠圆귚ᇟ赒ᡘ浚預鿹ᘓฑ圲肋ꕬ჆㓘륳쌫텮厬攞ᕇ䮽ꢗ牴쫚굣篁ж怏娈뭑싒樞ጡ矡鸉퉱㾼러⁩⨥ቭ桅做휚࠿멞㓧\uDA80䷸㠻ご砕紭䞏玆䪗ৰﰸ斺㯈璏ﶔꃙ剧뇗ވ㥋༣咨喘벷긳닅厒ᆻ唣퓽뾖跴퉈ㄳ⟵⼚셅쒱輎ᾟ笴㗸䩽\uDF42吓ꅘ軟᢭褶欲෗᪸蠬騻ꥼ籎䋾âꙷನ䮲蹼㗘㞑\uDD45ꦪ쮝乳頇ᘜ智⊴Ꟙ䍹♀뷿짿ꍵᲜ촾㉂냰騞Ҥ晲駀牵揄䤸䆳၅뿐䧨箧곟὚㫽揖쬨繥쨉딭㶲쁉㝓Հ濘⑙࢟鸀兊\uD881ꪨ줢ꁳ㎥哕ヅ䳓רּ㴤됈֕倬詗ʿ깗憭㼟᜗꭛욙⎅⑴ឮ窗'")
                .expect("Valid parse").sv(),
            vec![ 17028, 14861, 17735, 12870, 40569, 18705, 35634, 34377, 21913, 24122, 10990, 2778, 35620, 61404, 15731, 63376, 39612, 46876, 7402, 64606, 16389, 2018, 20862, 7927, 29025, 40753, 46777, 42326, 18235, 56479, 32066, 61027, 34452, 13308, 47848, 36567, 52208, 48560, 51475, 22478, 40003, 33195, 57595, 9014, 53991, 64660, 37275, 15971, 61616, 19701, 36384, 22278, 57883, 44506, 4575, 36178, 58627, 6232, 27994, 38928, 40953, 5651, 3601, 22322, 32907, 57580, 42348, 4294, 13528, 47475, 61491, 49963, 53614, 21420, 25886, 5447, 19389, 43159, 29300, 51930, 44387, 31681, 1078, 24591, 23048, 47953, 49874, 60619, 27166, 4897, 30689, 40457, 53873, 16316, 61713, 47084, 8297, 10789, 61059, 4717, 26693, 20570, 55066, 2111, 47710, 13543, 55936, 57772, 19960, 57799, 14395, 12372, 30741, 32045, 18319, 29574, 19095, 58510, 2544, 64568, 57946, 26042, 15304, 58185, 29839, 64916, 41177, 21095, 45527, 1928, 14667, 3875, 21672, 63164, 21912, 48311, 44595, 45765, 21394, 4539, 21795, 54525, 49046, 36340, 53832, 12595, 10229, 12058, 49477, 50353, 36622, 8095, 31540, 13816, 19069, 57154, 21523, 41304, 36575, 60038, 6317, 35126, 27442, 3543, 6840, 34860, 39483, 58493, 43388, 31822, 17150, 226, 42615, 3240, 19378, 36476, 13784, 14225, 56645, 43434, 52125, 20083, 38919, 5660, 26234, 8884, 42968, 17273, 9792, 48639, 51711, 41845, 7324, 52542, 12866, 45296, 39454, 1188, 26226, 39360, 29301, 64141, 18744, 57736, 16819, 4165, 62390, 59943, 49104, 18920, 31655, 44255, 8026, 15101, 25558, 52008, 32357, 51721, 46381, 15794, 49225, 14163, 1344, 28632, 9305, 2207, 40448, 60855, 61830, 20810, 55425, 43688, 57670, 51490, 41075, 13221, 21717, 12485, 19667, 64328, 15652, 46088, 1429, 20524, 35415, 703, 44631, 25005, 16159, 5911, 43867, 50841, 9093, 9332, 6062, 31383]
        );
    }
}
