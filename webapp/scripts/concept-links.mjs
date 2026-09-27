// Link the first mention of each related concept in plain text portions of a
// rendered post. Do not alter links, code, or markup supplied by the author.
export function linkConcepts(html, concepts) {
  if (!concepts.length) return html;
  const terms = new Map(concepts.flatMap((concept) =>
    [concept.name, ...concept.aliases].map((term) => [term.toLowerCase(), concept])));
  const names = [...terms.keys()].sort((a, b) => b.length - a.length)
    .map((name) => name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"));
  const pattern = new RegExp(`(?<![\\p{L}\\p{N}])(${names.join("|")})(?![\\p{L}\\p{N}])`, "giu");
  const linked = new Set();
  const blocked = ["a", "code", "pre", "script", "style"];
  let depth = 0;
  return html.split(/(<[^>]+>)/g).map((part) => {
    if (part.startsWith("<")) {
      const close = part.match(/^<\s*\/\s*([a-z]+)/i);
      const open = part.match(/^<\s*([a-z]+)/i);
      if (close && blocked.includes(close[1].toLowerCase())) depth--;
      if (open && blocked.includes(open[1].toLowerCase())) depth++;
      return part;
    }
    if (depth || !part.trim()) return part;
    return part.replace(pattern, (match) => {
      const concept = terms.get(match.toLowerCase());
      if (!concept || linked.has(concept.slug)) return match;
      linked.add(concept.slug);
      return `<a class="concept-term" href="/concepts#${concept.slug}">${match}</a>`;
    });
  }).join("");
}
