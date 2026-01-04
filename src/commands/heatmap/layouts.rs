use std::fmt::Display;
use strum::{EnumIter, IntoEnumIterator};

pub const KEY_HEIGHT: u16 = 3;

#[derive(Clone, Debug)]
pub struct KeyParams {
    pub label: String,
    pub json_key: String,
    pub x: u16,
    pub y: u16,
    pub width: u16,
}

impl KeyParams {
    pub fn new(label: &str, json_key: &str, x: u16, y: u16, width: u16) -> Self {
        Self {
            label: label.to_string(),
            json_key: json_key.to_string(),
            x,
            y,
            width,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum KeyboardLayout {
    Qwerty,
    Qwertz,
    Azerty,
    Dvorak,
    Colemak,
    Workman,
    Qzerty,
    DvorakLeft,
    DvorakRight,
    ProgrammerDvorak,
    ColemakModDh,
    Norman,
    Bepo,
    // New additions
    Neo,
    AdNW,
    Halmak,
    Engram,
    // New additions
    Mtgap,
    Capewell,
    CapewellDvorak,
    Qwerf,
    Minimak,
    Tarmak,
    CarpalxQgmlwy,
    CarpalxQwyrfm,
    Asset,
    Qwpr,
    Klauser,
    Arensito,
    HandsDown,
    Canary,
    Gallium,
    Semimak,
    Graphite,
    Sturdy,
    Ren,
    Isrt,
    Maltron,
    Malt,
    Hcesar,
    Fitaly,
    Jcuken,
    JcukenPhonetic,
    Arabic101,
    Arabic102,
    PersianStandard,
    Urdu,
    Pashto,
    HebrewStandard,
    HebrewPhonetic,
    TurkishF,
    TurkishQ,
    Greek,
    Inscript,
    Tamil99,
    Wijesekara,
    ThaiKedmanee,
    ThaiPattachote,
    Khmer,
    Lao,
    Myanmar,
    Vietnamese,
    Georgian,
    Armenian,
    Cherokee,
    Tifinagh,
    Inuktitut,
    Dzongkha,
    Tibetan,
    MongolianCyrillic,
    BulgarianPhonetic,
    UkrainianEnhanced,
    Belarusian,
    Kazakh,
    Scandinavian,
    SwissGerman,
    SwissFrench,
    CanadianMultilingual,
    UsInternational,
    UkExtended,
    Brazilian,
    Portuguese,
    Spanish,
    Italian,
    Latvian,
    LithuanianAzerty,
    Estonian,
    PolishProgrammers,
    RomanianProgrammers,
    CzechProgrammers,
    Hungarian,
}

impl Display for KeyboardLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyboardLayout::Qwerty => write!(f, "US QWERTY"),
            KeyboardLayout::Qwertz => write!(f, "German QWERTZ"),
            KeyboardLayout::Azerty => write!(f, "French AZERTY"),
            KeyboardLayout::Dvorak => write!(f, "Dvorak Standard"),
            KeyboardLayout::Colemak => write!(f, "Colemak"),
            KeyboardLayout::Workman => write!(f, "Workman"),
            KeyboardLayout::Qzerty => write!(f, "Italian QZERTY"),
            KeyboardLayout::DvorakLeft => write!(f, "Dvorak Left-Handed"),
            KeyboardLayout::DvorakRight => write!(f, "Dvorak Right-Handed"),
            KeyboardLayout::ProgrammerDvorak => write!(f, "Programmer Dvorak"),
            KeyboardLayout::ColemakModDh => write!(f, "Colemak Mod-DH"),
            KeyboardLayout::Norman => write!(f, "Norman"),
            KeyboardLayout::Bepo => write!(f, "French Bépo"),
            KeyboardLayout::Neo => write!(f, "German Neo 2"),
            KeyboardLayout::AdNW => write!(f, "AdNW (Aus der Neo-Welt)"),
            KeyboardLayout::Halmak => write!(f, "Halmak"),
            KeyboardLayout::Engram => write!(f, "Engram"),
            KeyboardLayout::Mtgap => write!(f, "MTGAP"),
            KeyboardLayout::Capewell => write!(f, "Capewell"),
            KeyboardLayout::CapewellDvorak => write!(f, "Capewell-Dvorak"),
            KeyboardLayout::Qwerf => write!(f, "QWERF"),
            KeyboardLayout::Minimak => write!(f, "Minimak"),
            KeyboardLayout::Tarmak => write!(f, "Tarmak"),
            KeyboardLayout::CarpalxQgmlwy => write!(f, "Carpalx QGMLWY"),
            KeyboardLayout::CarpalxQwyrfm => write!(f, "Carpalx QWYRFM"),
            KeyboardLayout::Asset => write!(f, "Asset"),
            KeyboardLayout::Qwpr => write!(f, "QWPR"),
            KeyboardLayout::Klauser => write!(f, "Klauser"),
            KeyboardLayout::Arensito => write!(f, "Arensito"),
            KeyboardLayout::HandsDown => write!(f, "Hands Down"),
            KeyboardLayout::Canary => write!(f, "Canary"),
            KeyboardLayout::Gallium => write!(f, "Gallium"),
            KeyboardLayout::Semimak => write!(f, "Semimak"),
            KeyboardLayout::Graphite => write!(f, "Graphite"),
            KeyboardLayout::Sturdy => write!(f, "Sturdy"),
            KeyboardLayout::Ren => write!(f, "Ren"),
            KeyboardLayout::Isrt => write!(f, "ISRT"),
            KeyboardLayout::Maltron => write!(f, "Maltron 3D"),
            KeyboardLayout::Malt => write!(f, "Malt"),
            KeyboardLayout::Hcesar => write!(f, "H-CESAR"),
            KeyboardLayout::Fitaly => write!(f, "FITALY"),
            KeyboardLayout::Jcuken => write!(f, "Russian JCUKEN"),
            KeyboardLayout::JcukenPhonetic => write!(f, "Russian Phonetic (YaWERT)"),
            KeyboardLayout::Arabic101 => write!(f, "Arabic (101)"),
            KeyboardLayout::Arabic102 => write!(f, "Arabic (102)"),
            KeyboardLayout::PersianStandard => write!(f, "Persian (Standard)"),
            KeyboardLayout::Urdu => write!(f, "Urdu"),
            KeyboardLayout::Pashto => write!(f, "Pashto"),
            KeyboardLayout::HebrewStandard => write!(f, "Hebrew (SI-1452)"),
            KeyboardLayout::HebrewPhonetic => write!(f, "Hebrew (Phonetic)"),
            KeyboardLayout::TurkishF => write!(f, "Turkish F"),
            KeyboardLayout::TurkishQ => write!(f, "Turkish Q"),
            KeyboardLayout::Greek => write!(f, "Greek"),
            KeyboardLayout::Inscript => write!(f, "Indian InScript"),
            KeyboardLayout::Tamil99 => write!(f, "Tamil 99"),
            KeyboardLayout::Wijesekara => write!(f, "Sinhala Wijesekara"),
            KeyboardLayout::ThaiKedmanee => write!(f, "Thai Kedmanee"),
            KeyboardLayout::ThaiPattachote => write!(f, "Thai Pattachote"),
            KeyboardLayout::Khmer => write!(f, "Khmer"),
            KeyboardLayout::Lao => write!(f, "Lao"),
            KeyboardLayout::Myanmar => write!(f, "Myanmar"),
            KeyboardLayout::Vietnamese => write!(f, "Vietnamese"),
            KeyboardLayout::Georgian => write!(f, "Georgian"),
            KeyboardLayout::Armenian => write!(f, "Armenian"),
            KeyboardLayout::Cherokee => write!(f, "Cherokee"),
            KeyboardLayout::Tifinagh => write!(f, "Tifinagh (Berber)"),
            KeyboardLayout::Inuktitut => write!(f, "Inuktitut (Nunavut)"),
            KeyboardLayout::Dzongkha => write!(f, "Dzongkha"),
            KeyboardLayout::Tibetan => write!(f, "Tibetan"),
            KeyboardLayout::MongolianCyrillic => write!(f, "Mongolian (Cyrillic)"),
            KeyboardLayout::BulgarianPhonetic => write!(f, "Bulgarian (Phonetic)"),
            KeyboardLayout::UkrainianEnhanced => write!(f, "Ukrainian (Enhanced)"),
            KeyboardLayout::Belarusian => write!(f, "Belarusian"),
            KeyboardLayout::Kazakh => write!(f, "Kazakh"),
            KeyboardLayout::Scandinavian => write!(f, "Nordic (Scandinavian)"),
            KeyboardLayout::SwissGerman => write!(f, "Swiss German"),
            KeyboardLayout::SwissFrench => write!(f, "Swiss French"),
            KeyboardLayout::CanadianMultilingual => write!(f, "Canadian Multilingual Standard"),
            KeyboardLayout::UsInternational => write!(f, "US International"),
            KeyboardLayout::UkExtended => write!(f, "UK Extended"),
            KeyboardLayout::Brazilian => write!(f, "Portuguese (Brazil ABNT2)"),
            KeyboardLayout::Portuguese => write!(f, "Portuguese (Portugal)"),
            KeyboardLayout::Spanish => write!(f, "Spanish (Spain)"),
            KeyboardLayout::Italian => write!(f, "Italian"),
            KeyboardLayout::Latvian => write!(f, "Latvian (QWERTY)"),
            KeyboardLayout::LithuanianAzerty => write!(f, "Lithuanian ĄŽERTY"),
            KeyboardLayout::Estonian => write!(f, "Estonian"),
            KeyboardLayout::PolishProgrammers => write!(f, "Polish (Programmers)"),
            KeyboardLayout::RomanianProgrammers => write!(f, "Romanian (Programmers)"),
            KeyboardLayout::CzechProgrammers => write!(f, "Czech (Programmers)"),
            KeyboardLayout::Hungarian => write!(f, "Hungarian"),
        }
    }
}

impl KeyboardLayout {
    pub fn get_keys(&self) -> Vec<KeyParams> {
        let map_str = match self {
            Self::Qwerty => "`1234567890-=qwertyuiop[]\\asdfghjkl;'zxcvbnm,./",
            Self::Qwertz => "^1234567890ß´qwertzuiopü+#asdfghjklöäyxcvbnm,.-",
            Self::Azerty => "²&é\"'(-è_çà)=azertyuiop^$*qsdfghjklmùwxcvbn,;:!",
            Self::Dvorak => "`1234567890[]',.pyfgcrl/=\\aoeuidhtns-;qjkxbmwvz",
            Self::Colemak => "`1234567890-=qwfpgjluy;[]\\arstdhneio'zxcvbkm,./",
            Self::Workman => "`1234567890-=qdrwbjfup;[]\\ashtgyneoi'zxmcvkl,./",
            Self::Qzerty => "\\1234567890'ìqzweyuiopè+ùasdfghjklmàwxcvbn.,-ò",
            Self::DvorakLeft => "`[]/pfmlj4321;qbyurso.65=\\-kcdtheaz87'xgwvni,90",
            Self::DvorakRight => "`1234567890/[]'q,.pyfglc=\\zaoeuidhtns-qjkxbmwvz",
            Self::ProgrammerDvorak => "$&[{}(=*)+]!#;,.pyfgcrl/@\\aoeuidhtns-'qjkxbmwvz",
            Self::ColemakModDh => "`1234567890-=qwfpbjluy;[]\\arstgmneio'zxcdvkh,./",
            Self::Norman => "`1234567890-=qwdfkjurlo;[]\\asetgynioh'zxcvbpm,./",
            Self::Bepo => "$1234567890=%b_po_v_dljzWçauie,ctsrnm_à_y.k'qghf",
            Self::Neo => "^1234567890-`xvlcwkhgfqß´uiaeosnrtdyüöäpzbm,.j",
            Self::AdNW => "`1234567890-=kuü.ävgcljf[]\\hieaodtrnsßxyö,qbpwmz",
            Self::Halmak => "`1234567890-=wlrbz;qud j[]\\shnt,aeoi'fmvc/g.kpx",
            Self::Engram => "`1234567890-=byou'ldwvz[]\\ciea,htsnqgxrmkjpf.;",

            // New additions
            Self::Mtgap => "`1234567890-=ypoujkdlcw[]\\inearsghtm;'qz/.,bfvx",
            Self::Capewell => "`1234567890-=.ywdfjpluq[]\\aersgbtnio;'xzvcmhk,/", // Approx
            Self::CapewellDvorak => "`1234567890[]'.,pyfgcrl/=\\aoeuidhtns-;qjkxbmwvz",
            Self::Qwerf => "`1234567890-=qwerfjuio;[]\\asdyghklnm'zxcvbp,./", // Placeholder
            Self::Minimak => "`1234567890-=qwdrykuio;[]\\asethgjnl'zxcvbpm,./", // Minimak 8
            Self::Tarmak => "`1234567890-=qweptjyuk;[]\\asdrglnioh'zxcvbmf,./", // Tarmak 1
            Self::CarpalxQgmlwy => "`1234567890-=qgmlwyfub;[]\\dstnriaeoh'zxcvjkp,./",
            Self::CarpalxQwyrfm => "`1234567890-=qwyrfmluob[]\\asdhtgneio'zxcvjkp,./",
            Self::Asset => "`1234567890-=qwjfgypul;[]\\asetdhnior'zxcvbkm,./",
            Self::Qwpr => "`1234567890-=qwprfyjuld[]\\asetghnio;'zxcvbkm,./",
            Self::Klauser => "`1234567890-=qwertyuiop[]\\asdfghjkl;'zxcvbnm,./", // Placeholder
            Self::Arensito => "`1234567890-=ql,p......[]\\arenbgsito;'kmhdfuvc.,/wxyz......", // Arensito
            Self::HandsDown => "`1234567890-=jgyu-l/.[;[]\\ristdhn,eo'xkbvwmqzf", // Hands Down Reference (ANSI)
            Self::Canary => "`1234567890-=wlypkzxou;[]\\crstbfneia'jvdgqm/.,",
            Self::Gallium => "`1234567890-=bldcvjyou,[]\\nrtsgphaei'xqmwzkf.;/",
            Self::Semimak => "`1234567890-=flhdmv,uo;[]\\srntkyaei.'wxgbqjzp/",
            Self::Graphite => "`1234567890-=bldwz'fouj[]\\nrtsgyhaei;qxmcvkp.,/",
            Self::Sturdy => "`1234567890-=vmlcpx.ou;[]\\stryknaeih'gwjdfbzq,/",
            Self::Ren => "`1234567890-=vymcuk.ou;[]\\strlhnai,e'gwjdfbzqp/",
            Self::Isrt => "`1234567890-=yclmkzfu,;[]\\isrtgpneao'qvwdjbh./x",
            Self::Maltron => "`1234567890-=qpycbvmuzl[]\\anisfdthoe'jwg,k.x;r", // ANSI approximation
            Self::Malt => "`1234567890-=qpycbvmuzl[]\\anisfdthoe'jwg,k.x;r", // Same as Maltron usually
            Self::Hcesar => "`1234567890-=hcesarodin[]\\tulpqgmbvf;'zjxkyw.,-/",
            Self::Jcuken => "ё1234567890-=йцукенгшщзхъ\\фывапролджэячсмитьбю.",
            Self::Arabic101 => "ذ1234567890-=ضصثقفغعهخحجد\\شسيبلاتنمكطئءؤرلاىةوزظ",
            Self::Arabic102 => "ذ1234567890-=ضصثقفغعهخحجد\\شسيبلاتنمكطئءؤرلاىةوزظ", // Very similar to 101, usually < > variations
            Self::PersianStandard => "÷1234567890-=ضصثقفغعهخحجچ\\شسيبلاتنمكگظطزرذدپو.",
            Self::Urdu => "ۓ1234567890-=ٹچپہجخگفدعثڑ\\مَنلتکیبشسوضقصرذڈزطظ.",
            Self::Pashto => "پ1234567890-=ٹڅچږجخگفدعښ\\مۍنلتکیبشسوضقصرذډزطظ.",
            Self::HebrewStandard => ";1234567890-=/'קראטוןםפ][\\שדגכעיחלךף,זסבהנמצתץ.",
            Self::HebrewPhonetic => "`1234567890-=קװארטיופ][\\אסדפגהJKL;'זXCVBנמ,./", // Rough phonetic
            Self::TurkishF => "+1234567890/-fgğıodrnhpqwxuieaütkmlyşjövcçzsb.,",
            Self::TurkishQ => "\"1234567890*-qwertyuıopğü,asdfghjklşi<zxcvbnmöç.",
            Self::Greek => "`1234567890-=;ςερτυθιοπ[]\\ασδφγηξκλ΄'ζχψωβνμ,./",
            Self::Inscript => "ॊ1234567890-=ौैाीूबहगदजड़\\ोे्िुपरकतचटॆंमनलसवशयष",
            Self::Tamil99 => "ஆ1234567890-=ஆஈஊஐஏளரனடண\\ஓஏஅஇஉபகதசஜொோ்ிுயலறவஷ",
            Self::Wijesekara => "1234567890-=ුඅැරඑහිසදච[]\\්ිාෙටයවනක;'ංජඩඉබප,./",
            Self::ThaiKedmanee => "_1234567890-=ภถุึคตจขชๆไ\\ๆไำพะัีรนยบฃฟหกดเ้่าสวง",
            Self::ThaiPattachote => "1234567890-=ตยอรรนวมงล\\กล่ดกเ้่าสบปอทมใฝ", // Approx
            Self::Khmer => "1234567890-=ឆึេរតយុិោព[]\\ាសដថងហ្កល;'ឋខចវបនម,./",
            Self::Lao => "ຢຟໂຖຸູຄຕຈຂ-=ົາເີືແ ້ ັ ືຍ[]\\ັຫກດເ ້ ່າສ;'ຜປແອຶືທ,./",
            Self::Myanmar => "၁၂၃၄၅၆၇၈၉၀-=ဆတနမအပကငသစ[]\\ေျိ်ါ့ြုူ;'ဖထခလဘညာ,./",
            Self::Vietnamese => "ĂÂÊỘ̀̉̃́Đ₫_qwertyuiopƯƠ\\asdfghjkl;'zxcvbnm,./",

            // Missing Map Strings
            Self::Fitaly => "1234567890-=zvchwk......[]\\fitaly......;'gdorsb......,./qjumpx......",
            Self::JcukenPhonetic => "~1234567890-=явертыуиоп[]\\асдфгхйкл;'зхьцвбнм,./",
            Self::Georgian => "“1234567890-=ქწერთყუიოპ[]\\ასდფგჰჯკლ;'ზხცვბნმ,./",
            Self::Armenian => "՝1234567890-=քոեռթըւիօպ[]\\ասդֆգհյկլ;'զղցվբնմ,./",
            Self::Cherokee => "Ꮚ1234567890-=ᏯᏪᎡᏛᎢᏲᎤᎢᎣᏢ[]\\ᎠᏍᏓᏩᎦᎭᎫᎧᎸ;'ᏴᏟᏟᏭᏄᎹ,./",
            Self::Tifinagh => "²1234567890-=ⴰⵣⴻⵔⵜⵢⵓⵉⵄⵃ[]\\ⵇⵙⴷⴼⴳⵀⵊⴽⵍⵎ;'ⵡⵅⵛⵯⴱⵏ,;:!",
            Self::Inuktitut => "1234567890-=qwertyuiop[]\\asdfghjkl;'zxcvbnm,./", // Placeholder
            Self::Dzongkha => "1234567890-=qwertyuiop[]\\asdfghjkl;'zxcvbnm,./",  // Placeholder
            Self::Tibetan => "1234567890-=qwertyuiop[]\\asdfghjkl;'zxcvbnm,./",   // Placeholder
            Self::MongolianCyrillic => "№1234567890-=йцукенгшщзхъ\\фывапролджэ.ячсмитьбю,",
            Self::BulgarianPhonetic => "ю1234567890-=чшертъуиопящ\\асдфгхйкл;'жзьцвбнм,./",
            Self::UkrainianEnhanced => "'1234567890-=йцукенгшщзхї\\фывапролджє/ячсмитьбю.",
            Self::Belarusian => "ё1234567890-=йцукенгшўзх'\\фывапролджэ/ячсмітьбю.",
            Self::Kazakh => "(1234567890-=йцукенгшщзхъ\\фывапролджэ/ячсмитьбю.",
            Self::Scandinavian => "§1234567890+´qwertyuiopå¨'asdfghjklöä<zxcvbnm,.-",
            Self::SwissGerman => "§1234567890'^qwertzuiopü¨$asdfghjklöä<yxcvbnm,.-",
            Self::SwissFrench => "§1234567890'^qwertzuiopè¨$asdfghjkléà<yxcvbnm,.-",
            Self::CanadianMultilingual => "/1234567890-=\\qwertyuiop^¸[asdfghjkl;`]zxcvbnm,./",
            Self::UsInternational => "`1234567890-=qwertyuiop[]\\asdfghjkl;'zxcvbnm,./",
            Self::UkExtended => "`1234567890-=qwertyuiop[]#asdfghjkl;'\\zxcvbnm,./",
            Self::Brazilian => "'1234567890-=qwertyuiop´[}asdfghjklç^]zxcvbnm,.;/",
            Self::Portuguese => "\\1234567890'«qwertyuiop+´~asdfghjklçº<zxcvbnm,.-",
            Self::Spanish => "º1234567890'¡qwertyuiop`+çasdfghjklñ´<zxcvbnm,.-",
            Self::Italian => "\\1234567890'ìqwertyuiopè+asdfghjklòàù<zxcvbnm,.-",
            Self::Latvian => "`1234567890-=qwertyuiop[]\\asdfghjkl;'zxcvbnm,./",
            Self::LithuanianAzerty => "ĄČĘĖĮŠŲŪ90-=Žqwertyuiop[]\\asdfghjkl;'zxcvbnm,./",
            Self::Estonian => "ˇ1234567890+´qwertyuiopüõ'asdfghjklöä<zxcvbnm,.-",
            Self::PolishProgrammers => "~1234567890+=qwertyuiop[]\\asdfghjkl;'zxcvbnm,./",
            Self::RomanianProgrammers => "~1234567890+=qwertyuiop[]\\asdfghjkl;'zxcvbnm,./",
            Self::CzechProgrammers => "~1234567890+=qwertyuiop[]\\asdfghjkl;'zxcvbnm,./",
            Self::Hungarian => "0123456789öüóqwertzuiopőúűasdfghjkléáíyxcvbnm,.-",
        };

        // Sanity check for string length if needed, but we'll just process what we have
        build_ansi_layout(map_str)
    }

    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }
}

fn build_ansi_layout(map: &str) -> Vec<KeyParams> {
    let mut keys = Vec::new();
    let chars: Vec<char> = map.chars().collect();

    // Helper to add key
    let mut add = |label: &str, json: &str, x: u16, y: u16, w: u16| {
        keys.push(KeyParams::new(label, json, x, y, w));
    };

    // --- Row 1 ---
    // Physical: Grave, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, Minus, Equal
    let r1_x = [0, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 44, 48];
    for i in 0..13 {
        if i < chars.len() {
            let label = chars[i].to_string();
            // Special handling for uppercase if it's a letter
            let display_label = if label.chars().next().unwrap().is_alphabetic() {
                label.to_uppercase()
            } else {
                label.clone()
            };
            let json_key = get_api_key_from_char(chars[i]).to_uppercase();
            add(&display_label, &json_key, r1_x[i], 0, 4);
        }
    }
    // Backspace (Fixed)
    add("Bksp", "BACKSPACE", 52, 0, 8);

    // --- Row 2 ---
    // Tab (Fixed)
    add("Tab", "TAB", 0, 3, 6);
    // Physical: Q, W, E, R, T, Y, U, I, O, P, BracketLeft, BracketRight, Backslash
    let r2_x = [6, 10, 14, 18, 22, 26, 30, 34, 38, 42, 46, 50, 54];
    for i in 0..13 {
        if 13 + i < chars.len() {
            let label = chars[13 + i].to_string();
            let display_label = if label.chars().next().unwrap().is_alphabetic() {
                label.to_uppercase()
            } else {
                label.clone()
            };
            let json_key = get_api_key_from_char(chars[13 + i]).to_uppercase();
            add(
                &display_label,
                &json_key,
                r2_x[i],
                3,
                if i == 12 { 6 } else { 4 },
            );
        }
    }

    // --- Row 3 ---
    // Caps (Fixed)
    add("Caps", "CAPSLOCK", 0, 6, 7);
    // Physical: A, S, D, F, G, H, J, K, L, Semicolon, Apostrophe
    let r3_x = [7, 11, 15, 19, 23, 27, 31, 35, 39, 43, 47];
    for i in 0..11 {
        if 26 + i < chars.len() {
            let label = chars[26 + i].to_string();
            let display_label = if label.chars().next().unwrap().is_alphabetic() {
                label.to_uppercase()
            } else {
                label.clone()
            };
            let json_key = get_api_key_from_char(chars[26 + i]).to_uppercase();
            add(&display_label, &json_key, r3_x[i], 6, 4);
        }
    }
    // Enter (Fixed)
    add("Enter", "RETURN", 51, 6, 9);

    // --- Row 4 ---
    // Left Shift (Fixed)
    add("Shift", "LEFTSHIFT", 0, 9, 9);
    // Physical: Z, X, C, V, B, N, M, Comma, Period, Slash
    let r4_x = [9, 13, 17, 21, 25, 29, 33, 37, 41, 45];
    for i in 0..10 {
        if 37 + i < chars.len() {
            let label = chars[37 + i].to_string();
            let display_label = if label.chars().next().unwrap().is_alphabetic() {
                label.to_uppercase()
            } else {
                label.clone()
            };
            let json_key = get_api_key_from_char(chars[37 + i]).to_uppercase();
            add(&display_label, &json_key, r4_x[i], 9, 4);
        }
    }
    // Right Shift (Fixed)
    add("Shift", "RIGHTSHIFT", 49, 9, 11);

    // --- Row 5 ---
    // Fixed Control row
    add("Ctrl", "LEFTCONTROL", 0, 12, 5);
    add("Win", "LEFTWINDOWS", 5, 12, 5);
    add("Alt", "LEFTALT", 10, 12, 5);
    add("Space", "SPACE", 15, 12, 25);
    add("Alt", "RIGHTALT", 40, 12, 5);
    add("Win", "RIGHTWINDOWS", 45, 12, 5);
    add("Menu", "MENU", 50, 12, 5);
    add("Ctrl", "RIGHTCONTROL", 55, 12, 5);

    keys
}

fn get_api_key_from_char(c: char) -> String {
    match c.to_ascii_uppercase() {
        // Alphanumeric - these are already uppercase from to_ascii_uppercase
        'A'..='Z' => c.to_ascii_uppercase().to_string(),
        '0'..='9' => c.to_string(),

        // Standard Symbols
        '-' | '_' => "MINUS".to_string(),
        '=' | '+' => "EQUAL".to_string(),
        '[' | '{' => "BRACKETLEFT".to_string(),
        ']' | '}' => "BRACKETRIGHT".to_string(),
        '\\' | '|' => "BACKSLASH".to_string(),
        ';' | ':' => "SEMICOLON".to_string(),
        '\'' | '"' => "QUOTE".to_string(),
        ',' | '<' => "COMMA".to_string(),
        '.' | '>' => "PERIOD".to_string(),
        '/' | '?' => "SLASH".to_string(),
        '`' | '~' => "GRAVE".to_string(),
        ' ' => "SPACE".to_string(),

        // Shifted Numbers
        '!' => "1".to_string(),
        '@' => "2".to_string(),
        '#' => "3".to_string(),
        '$' => "4".to_string(),
        '%' => "5".to_string(),
        '^' => "6".to_string(),
        '&' => "7".to_string(),
        '*' => "8".to_string(),
        '(' => "9".to_string(),
        ')' => "0".to_string(),

        // Special / International
        'Ç' | 'ç' => "CEDILLA".to_string(),
        'Ñ' | 'ñ' => "NTILDE".to_string(),

        // Fallback
        other => other.to_string().to_uppercase(),
    }
}
