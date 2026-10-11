// Share cards: a 1200x630 image (the size social sites use for previews) drawn
// on a canvas, shared with the phone's share sheet or downloaded.
export type Card = { kicker: string; title: string; lines: string[]; file: string };

function wrap(ctx: CanvasRenderingContext2D, text: string, max: number): string[] {
  const words = text.split(" ");
  const out: string[] = [];
  let line = "";
  for (const w of words) {
    const next = line ? line + " " + w : w;
    if (ctx.measureText(next).width > max && line) {
      out.push(line);
      line = w;
    } else line = next;
  }
  if (line) out.push(line);
  return out;
}

export async function drawCard(card: Card): Promise<Blob> {
  const c = document.createElement("canvas");
  c.width = 1200;
  c.height = 630;
  const ctx = c.getContext("2d")!;
  const font = '"Inter Variable", ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto, Arial, sans-serif';
  await Promise.all([document.fonts.load(`700 40px "Inter Variable"`), document.fonts.load(`800 68px "Inter Variable"`)]).catch(() => {});
  ctx.fillStyle = "#0f0b09";
  ctx.fillRect(0, 0, 1200, 630);
  const glow = ctx.createRadialGradient(1000, 150, 0, 1000, 150, 620);
  glow.addColorStop(0, "rgba(247, 76, 0, 0.24)");
  glow.addColorStop(1, "rgba(247, 76, 0, 0)");
  ctx.fillStyle = glow;
  ctx.fillRect(0, 0, 1200, 630);

  // The logo (as in components/Brand.tsx), drawn at twice its 32px size.
  ctx.save();
  ctx.translate(80, 72);
  ctx.scale(2, 2);
  const tile = ctx.createLinearGradient(0, 0, 32, 32);
  tile.addColorStop(0, "#F98A1F");
  tile.addColorStop(0.5, "#E5531A");
  tile.addColorStop(1, "#B7410E");
  ctx.fillStyle = tile;
  ctx.beginPath();
  ctx.roundRect(0, 0, 32, 32, 8);
  ctx.fill();
  ctx.fillStyle = "#fff";
  ctx.strokeStyle = "#fff";
  ctx.lineCap = "round";
  ctx.lineJoin = "round";
  ctx.fill(new Path2D("M5.2 15.4c-1.7-1.3-1.9-4-.4-5.6 1-1 2.4-1.2 3.6-.7l-1.3 2.3 2.4.4c.3 1.6-.4 3.2-1.8 3.9-.8.4-1.7.3-2.5-.3z"));
  ctx.fill(new Path2D("M26.8 15.4c1.7-1.3 1.9-4 .4-5.6-1-1-2.4-1.2-3.6-.7l1.3 2.3-2.4.4c-.3 1.6.4 3.2 1.8 3.9.8.4 1.7.3 2.5-.3z"));
  ctx.lineWidth = 1.6;
  ctx.stroke(new Path2D("M8.6 15.2l2.4 2.2M23.4 15.2l-2.4 2.2"));
  ctx.lineWidth = 1.5;
  ctx.stroke(new Path2D("M10.2 22.2l-2 2.6M12.6 23.2l-1.2 2.8M21.8 22.2l2 2.6M19.4 23.2l1.2 2.8"));
  const body = new Path2D("M8.6 20.4c0-3.9 3.3-7 7.4-7s7.4 3.1 7.4 7c0 1.9-3.3 3.3-7.4 3.3s-7.4-1.4-7.4-3.3zM9.6 16.6l.4-2.2 1.6 1M11.4 14.6l.9-2.1 1.3 1.4M14 13.4l1-1.9 1 1.9M17 13.4l1.3-1.4.9 2.1M19.4 14.6l1.6-1 .4 2.2");
  ctx.fill(body);
  ctx.lineWidth = 1;
  ctx.stroke(body);
  ctx.fillStyle = "#2b1206";
  for (const x of [13.3, 18.7]) {
    ctx.beginPath();
    ctx.arc(x, 18.6, 1.35, 0, Math.PI * 2);
    ctx.fill();
  }
  ctx.restore();
  ctx.fillStyle = "#f7f1ec";
  ctx.font = `700 40px ${font}`;
  ctx.fillText("RustArena", 166, 118);

  ctx.fillStyle = "#FF9A5C";
  ctx.font = `700 28px ${font}`;
  ctx.fillText(card.kicker.toUpperCase(), 80, 240);
  ctx.fillStyle = "#fbf6f2";
  ctx.font = `750 68px ${font}`;
  const titleLines = wrap(ctx, card.title, 1040).slice(0, 2);
  titleLines.forEach((l, i) => ctx.fillText(l, 80, 320 + i * 78));
  ctx.fillStyle = "#e2d6cd";
  ctx.font = `500 32px ${font}`;
  card.lines.slice(0, 2).forEach((l, i) => ctx.fillText(l, 80, 320 + titleLines.length * 78 + 20 + i * 46));
  ctx.fillStyle = "#b3a296";
  ctx.font = `600 30px ${font}`;
  ctx.fillText("RustArena  ·  learn Rust from zero to job-ready", 80, 580);
  return new Promise((resolve, reject) => c.toBlob((b) => (b ? resolve(b) : reject(new Error("Couldn't draw the card"))), "image/png"));
}

/** Share the card (phone share sheet) or download it. Returns what happened, for a status message. */
export async function shareCard(card: Card, text: string): Promise<"shared" | "downloaded"> {
  const blob = await drawCard(card);
  const file = new File([blob], card.file, { type: "image/png" });
  const url = location.origin + import.meta.env.BASE_URL;
  if (navigator.canShare?.({ files: [file] })) {
    try {
      await navigator.share({ files: [file], text: `${text} ${url}` });
      return "shared";
    } catch (e) {
      if ((e as Error).name === "AbortError") return "shared";
    }
  }
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob);
  a.download = card.file;
  a.click();
  setTimeout(() => URL.revokeObjectURL(a.href), 5000);
  return "downloaded";
}
