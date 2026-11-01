// Rail model for the folder picker.
//
// The rail adapts to the library (D28): only writing systems actually present get a
// section, and within a section the cells are single letters / kana rows. There is
// deliberately NO sub-letter segmentation — the CA / CH / CL–CO split is dead (D17),
// in every presentation, including a rail accordion.
import { scriptOf, SCRIPT_LABEL } from "./artists.js";

const KANA_ROWS = [
  ["あ", "あいうえおアイウエオぁぃぅぇぉァィゥェォ"],
  ["か", "かきくけこカキクケコがぎぐげごガギグゲゴ"],
  ["さ", "さしすせそサシスセソざじずぜぞザジズゼゾ"],
  ["た", "たちつてとタチツテトだぢづでどダヂヅデドっッ"],
  ["な", "なにぬねのナニヌネノ"],
  ["は", "はひふへほハヒフヘホばびぶべぼバビブベボぱぴぷぺぽパピプペポ"],
  ["ま", "まみむめもマミムメモ"],
  ["や", "やゆよヤユヨゃゅょャュョ"],
  ["ら", "らりるれろラリルレロ"],
  ["わ", "わをんワヲンー"]
];

function cellKeyFor(name, script) {
  if (script === "latin") return name[0].toUpperCase();
  if (script === "jpn") {
    const c = name[0];
    for (const [label, chars] of KANA_ROWS) if (chars.indexOf(c) >= 0) return label;
    return "漢";
  }
  if (script === "cyrl" || script === "grek" || script === "hang") return name[0].toUpperCase();
  return "#";
}

export function railFor(names) {
  const scripts = new Map();
  names.forEach((n) => {
    const s = scriptOf(n);
    if (!scripts.has(s)) scripts.set(s, []);
    scripts.get(s).push(n);
  });
  const order = ["latin", "jpn", "cyrl", "grek", "hang", "other"].filter((s) => scripts.has(s));
  return order.map((s) => {
    const cells = new Map();
    scripts.get(s).forEach((n) => {
      const k = cellKeyFor(n, s);
      if (!cells.has(k)) cells.set(k, []);
      cells.get(k).push(n);
    });
    const keys = [...cells.keys()].sort((a, b) => a.localeCompare(b, undefined, { sensitivity: "base" }));
    return {
      script: s,
      label: SCRIPT_LABEL[s] || "#",
      cells: keys.map((k) => ({ key: s + k, label: k, names: cells.get(k) }))
    };
  });
}
