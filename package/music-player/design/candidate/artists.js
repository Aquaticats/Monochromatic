const SEEDS = [
  "32ki","3LAU","40mP","96neko","ABM","ACCAMER","Adele","Ado","Afrojack","Aimer","Aiobahn",
  "Aitsuki Nakuru","Alan Walker","Alessia Cara","Alesso","AREA21","Armin van Buuren",
  "Asami Shimoda","Aventry","Avicii","Avril Lavigne","Axero","Axwell Ingrosso","Ayase",
  "Bassjackers","Bebe Rexha","Beyonce","BEYOND","Blank Dream RPG","Bon Jovi","Boyhood's end OST",
  "Broiler","Brooks","Bruno Mars","Bump of Chicken","Calvin Harris","Camellia","Charlie Puth",
  "Charon","Chris Medina","ClariS","Clean Bandit","Coldplay","Cosmo Bosou P","CreepP","CrusherP",
  "Cyberpunk 2077 OST","Daft Punk","daniwellP","Dash Berlin","David Guetta","DECO_27","Demi Lovato",
  "Detroit Become Human OST","Dimitri Vegas & Like Mike","Divela","DJ Genki","DJ Noriken","DJ Snake",
  "Doja Cat","Dua Lipa","Ed Sheeran","Eminem","Eve","EGOIST","Ellie Goulding","Eir Aoi","Elgar",
  "Fall Out Boy","Feint","Flume","Fox Stevenson","Frums","Fujii Kaze","Galantis","Getty","Gorillaz",
  "Gramatik","Grimes","Halsey","Hardwell","Hatsune Miku","Hikaru Utada","Hiroyuki Sawano","Hoshimachi",
  "Ice Nine Kills","Illenium","Imagine Dragons","Ina","Inabakumori","Jack Stauber","James Blake",
  "Jamie xx","Jason Derulo","Jay Chou","Joji","Justice","Kanaria","Kaskade","Katy Perry","Kenshi Yonezu",
  "Kero Kero Bonito","Kikuo","King Gnu","Knife Party","Kobaryo","Kygo","Lady Gaga","Lana Del Rey",
  "Laur","LiSA","Lorde","Louie Zong","Lovely Summer-chan","Ludwig Goransson","Maroon 5","Marshmello",
  "Martin Garrix","Masayoshi Oishi","MEIKO","Metroid Dread OST","Milet","Mitski","Mr Kitty","MSI",
  "Nanahira","Nekomata Master","Nier OST","Nightwish","Nujabes","Ochame Kinou","OneRepublic","Onoken",
  "Orangestar","Oshi no Ko OST","Paramore","Persona 5 OST","Pharrell","Pinocchio-P","Porter Robinson",
  "Post Malone","Powerless","Queen","Quruli","Radiohead","Reol","Rezz","Rin Kagamine","Rita Ora",
  "Rolling Girl","Sakanaction","Sam Smith","San Holo","Sasakure UK","Seatbelts","Sheena Ringo",
  "Shingo Nakamura","Sia","Silent Hill OST","Skrillex","Slushii","Snail's House","Sohn","Sonic Mania OST",
  "Susumu Hirasawa","Taku Inoue","Taylor Swift","Tears for Fears","Teminite","The Chainsmokers",
  "The Weeknd","Tiesto","Tohoku Kiritan","Tokyo Machine","Toby Fox","Tomggg","Twenty One Pilots",
  "Undertale OST","Unison Square Garden","Uru","Vaundy","Vicetone","Virtual Riot","Vocaloid Legacy",
  "Wagakki Band","Waterflame","Wednesday Campanella","Weezer","Whethan","Wiz Khalifa","Xilent",
  "xi","Xtrullor","Yasuharu Takanashi","Yoasobi","Yorushika","Yosh","Yuki Kajiura","Yunomi",
  "Zedd","Zeta","Zutomayo","ZZ Top"
];

// Real libraries are not Latin-only. These bucket into a single "#" cell under the
// old scheme, which is wrong for anyone whose library is mostly one of these scripts.
const NON_LATIN = [
  "かめりあ", "米津玄師", "ずっと真夜中でいいのに。", "東京事変", "椎名林檎", "サカナクション",
  "アイマス", "初音ミク", "菅野よう子", "平沢進", "きくお", "ヨルシカ", "あいみょん", "宇多田ヒカル",
  "上北健", "クレイジーケンバンド", "サンボマスター", "スピッツ", "タルトタタン", "ナナヒラ",
  "Ария", "Гражданская Оборона", "Кино", "Мумий Тролль", "Сплин", "Земфира", "ДДТ",
  "Ελευθερία Αρβανιτάκη", "Βασίλης Παπακωνσταντίνου", "이하이", "아이유", "방탄소년단"
];

// The library is ~1000 folders. Multiply the seeds so the candidates
// scroll at true scale rather than demo scale.
export const ARTISTS = (function () {
  const out = SEEDS.concat(NON_LATIN);
  const suffixes = ["", " II", " (live)", " Remixes", " EP", " OST"];
  for (let s = 1; s < suffixes.length; s++) {
    SEEDS.forEach(function (n) { out.push(n + suffixes[s]); });
  }
  return out.sort(function (a, b) {
    return a.localeCompare(b, undefined, { sensitivity: "base" });
  });
})();

export function bucketOf(name) {
  const c = name[0].toUpperCase();
  return c >= "A" && c <= "Z" ? c : "#";
}

// Which writing system a folder name starts with. Only scripts actually present in a
// library get a section in the rail, so a Latin-only library never sees あ or А.
export function scriptOf(name) {
  const c = name.codePointAt(0);
  if ((c >= 65 && c <= 90) || (c >= 97 && c <= 122) || (c >= 0xC0 && c <= 0x24F)) return "latin";
  if ((c >= 0x3040 && c <= 0x30FF) || (c >= 0x4E00 && c <= 0x9FFF) || (c >= 0xFF66 && c <= 0xFF9D)) return "jpn";
  if (c >= 0x400 && c <= 0x4FF) return "cyrl";
  if (c >= 0x370 && c <= 0x3FF) return "grek";
  if (c >= 0xAC00 && c <= 0xD7AF) return "hang";
  return "other";
}

export const SCRIPT_LABEL = { latin: "A", jpn: "あ", cyrl: "А", grek: "Α", hang: "가", other: "#" };

export const LETTERS = ["#"].concat("ABCDEFGHIJKLMNOPQRSTUVWXYZ".split(""));

export function grouped() {
  const map = new Map();
  LETTERS.forEach(function (l) { map.set(l, []); });
  ARTISTS.forEach(function (n) { map.get(bucketOf(n)).push(n); });
  return LETTERS.map(function (l) {
    return { letter: l, names: map.get(l), count: map.get(l).length };
  }).filter(function (g) { return g.count > 0; });
}
