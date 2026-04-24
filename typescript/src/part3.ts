import readline from "readline";

const RST  = "\x1b[0m";
const BOLD = "\x1b[1m";
const DIM  = "\x1b[2m";
const RED  = "\x1b[91m";
const GRN  = "\x1b[92m";
const YLW  = "\x1b[93m";
const BLU  = "\x1b[94m";
const MAG  = "\x1b[95m";
const CYN  = "\x1b[96m";
const GRY  = "\x1b[90m";

function codeOpen(lang: string, col: string) {
  console.log(`${DIM}  ╭─ ${RST}${BOLD}${col}${lang}${RST} ${DIM}────────────────────────────────────────────────────`);
  console.log(`  │${RST}`);
}
const codeOpenTs = () => codeOpen("typescript", BLU);
function codeClose() {
  console.log(`${DIM}  │`);
  console.log(`  ╰──────────────────────────────────────────────────────────${RST}\n`);
}
function sectionResult() {
  console.log(`${BOLD}${CYN}\n  ━━ Result ${RST}${DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RST}\n`);
}
function sectionExplain() {
  console.log(`${BOLD}${MAG}\n  ━━ What happened ${RST}${DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RST}\n`);
}

const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
let rlClosed = false;
rl.on("close", () => { rlClosed = true; });

async function pressEnter(question: string): Promise<void> {
  console.log(`${BOLD}${YLW}\n  ╔═ ? ════════════════════════════════════════════════════════╗${RST}`);
  console.log(`${BOLD}${YLW}  ║${RST}  ${question}`);
  console.log(`${BOLD}${YLW}  ║${RST}`);
  console.log(`${BOLD}${YLW}  ║${RST}  ${DIM}Press Enter to continue...${RST}`);
  console.log(`${BOLD}${YLW}  ╚════════════════════════════════════════════════════════════╝${RST}`);
  if (rlClosed) return;
  await new Promise<void>(resolve => rl.once("line", () => { console.log(); resolve(); }));
}

const nsNow = (): bigint => process.hrtime.bigint();
const p = () => `  ${DIM}│${RST}  `;

// ══════════════════════════════════════════════════════════════════
// Demo 1 — shared ownership (verbosity)
// ══════════════════════════════════════════════════════════════════
async function demoSharedOwnership(): Promise<void> {
  const pp = p();

  console.log(`  In TypeScript, sharing is just ${BLU}assignment${RST}. The GC traces all`);
  console.log(`  live references and frees memory in batches when it decides to:\n`);

  codeOpenTs();
  console.log(`${pp}${YLW}interface${RST} ${GRN}Node${RST} { value: ${YLW}number${RST} }`);
  console.log(`${DIM}  │${RST}`);
  console.log(`${pp}${YLW}const${RST} a: ${GRN}Node${RST} = { value: ${MAG}42${RST} };   ${GRY}// heap alloc — GC owns it${RST}`);
  console.log(`${pp}${YLW}const${RST} b = a;                 ${GRY}// aliases the same object${RST}`);
  console.log(`${pp}console.log(a.value, b.value); ${GRY}// both work${RST}`);
  console.log(`${pp}${GRY}// GC frees when no refs remain — timing is not your call${RST}`);
  codeClose();

  await pressEnter("Zero boilerplate — but what does TypeScript actually give you here?");
  sectionResult();

  interface Node { value: number }
  const a: Node = { value: 42 };
  const b = a;
  console.log(`  a.value=${GRN}${BOLD}${a.value}${RST}  b.value=${GRN}${BOLD}${b.value}${RST}  (same object in memory)`);
  b.value = 99;
  console.log(`  After b.value = 99: a.value=${YLW}${BOLD}${a.value}${RST}  ← a sees the mutation\n`);

  console.log(`  ${BOLD}Lines of boilerplate to safely share one heap value:\n${RST}`);
  console.log(`  ${RED}C (manual refcount)${RST}  ~15 lines  ${RED}███████████████${RST}`);
  console.log(`  ${GRN}Rust (Rc<T>)${RST}          4 lines  ${GRN}████${RST}`);
  console.log(`  ${BLU}TypeScript (GC)${RST}        0 lines  ${DIM}(GC tracks references invisibly)${RST}`);

  sectionExplain();
  console.log(`  TypeScript has no ownership — ${BLU}b = a${RST} is ${BOLD}aliasing${RST}, not a ref-count.`);
  console.log(`  The GC traces all reachable objects and frees them in batches.\n`);
  console.log(`  The ${YLW}trade-off${RST}:`);
  console.log(`    ${GRN}•${RST} Zero verbosity — sharing is natural and invisible`);
  console.log(`    ${RED}•${RST} Mutations through any alias affect all holders (shown above)`);
  console.log(`    ${RED}•${RST} No control over ${YLW}when${RST} memory is freed — see demo 2\n`);
  console.log(`  ${GRY}In Rust, Rc<T> gives shared ${BOLD}immutable${GRY} access; mutation`);
  console.log(`  ${GRY}requires explicit ${BOLD}RefCell<T>${GRY} — the compiler enforces the distinction.${RST}`);
}

// ══════════════════════════════════════════════════════════════════
// Demo 2 — allocation latency benchmark
// ══════════════════════════════════════════════════════════════════
const BATCH_SIZE  = 1000;
const BATCH_COUNT = 500;

interface Node { a: number; b: number; c: number; d: number; e: number; f: number }

async function demoAllocLatency(): Promise<void> {
  const pp = p();

  console.log(`  Each batch: ${YLW}${BATCH_SIZE}${RST} object allocations — the GC collects them later.`);
  console.log(`  Total: ${YLW}500,000${RST} allocations across ${YLW}${BATCH_COUNT}${RST} batches.\n`);

  codeOpenTs();
  console.log(`${pp}${YLW}const${RST} nodes: ${GRN}Node[]${RST} = ${YLW}new${RST} Array(BATCH_SIZE);`);
  console.log(`${pp}${YLW}for${RST} (${YLW}let${RST} i = 0; i < BATCH_SIZE; i++)`);
  console.log(`${pp}    nodes[i] = { a: i, b: i*2, ${GRY}/* ... */${RST} };`);
  console.log(`${pp}${GRY}// nodes goes out of scope — GC frees it when it decides to${RST}`);
  codeClose();

  await pressEnter("The GC accumulates garbage between batches. Will p95 differ from p50?");
  sectionResult();

  const times: number[] = new Array(BATCH_COUNT);
  let checksum = 0;

  for (let b = 0; b < BATCH_COUNT; b++) {
    const filled = Math.floor((b + 1) * 38 / BATCH_COUNT);
    const bar = GRN + "▓".repeat(filled) + RST + DIM + "░".repeat(38 - filled) + RST;
    process.stdout.write(`\r  ${DIM}[${RST}${bar}${DIM}]${RST}  ${b + 1}/${BATCH_COUNT}   `);

    const t0 = nsNow();
    const nodes: Node[] = new Array(BATCH_SIZE);
    for (let i = 0; i < BATCH_SIZE; i++) {
      nodes[i] = { a: i, b: i*2, c: i*3, d: i*4, e: i*5, f: i*6 };
    }
    for (const n of nodes) checksum += n.a;
    times[b] = Number(nsNow() - t0) / 1000; /* µs */
  }
  process.stdout.write(`\r  ${GRN}Complete.${RST}                                              \n\n`);

  times.sort((a, b) => a - b);
  const p50  = times[Math.floor(BATCH_COUNT / 2)];
  const p95  = times[Math.floor(BATCH_COUNT * 0.95)];
  const tmax = times[BATCH_COUNT - 1];
  const sc   = Math.max(tmax, 1);

  const bar = (n: number) => "█".repeat(Math.floor(n * 30 / sc));
  console.log(`  ${BOLD}Batch time (${BATCH_COUNT} batches × ${BATCH_SIZE} allocs):${RST}\n`);
  console.log(`   p50  ${GRN}${BOLD}${p50.toFixed(0).padStart(5)}${RST}µs  ${GRN}${bar(p50)}${RST}`);
  console.log(`   p95  ${YLW}${BOLD}${p95.toFixed(0).padStart(5)}${RST}µs  ${YLW}${bar(p95)}${RST}`);
  console.log(`   max  ${RED}${BOLD}${tmax.toFixed(0).padStart(5)}${RST}µs  ${RED}${bar(tmax)}${RST}`);
  console.log(`\n  ${DIM}(checksum=${checksum} — prevents dead-code elimination)${RST}`);

  sectionExplain();
  if (p95 > p50 * 2.5) {
    console.log(`  ${RED}${BOLD}GC pause visible:${RST} p95 is ${(p95 / p50).toFixed(1)}× p50.`);
    console.log(`  The GC swept accumulated garbage during at least one batch.\n`);
  } else {
    console.log(`  ${YLW}No large GC spike this run${RST} — JSC may have collected`);
    console.log(`  during your keypress pauses. Try running non-interactively:\n`);
    console.log(`  ${DIM}  printf '2\\n\\nq\\n' | bun run src/part3.ts${RST}\n`);
  }
  console.log(`  The GC runs ${YLW}non-deterministically${RST}. When it fires, it pauses`);
  console.log(`  the entire process — that spike appears in ${RED}p95 and max${RST}.\n`);
  console.log(`  C and Rust call free()/drop() ${BOLD}immediately${RST} per batch.`);
  console.log(`  No accumulation → ${GRN}p50 ≈ p95 ≈ max${RST}: flat, predictable latency.\n`);
  console.log(`  ${YLW}For most applications this doesn't matter.${RST}`);
  console.log(`  For game loops, real-time audio, or trading systems: it does.`);
}

// ─── menu ─────────────────────────────────────────────────────────────
async function printMenu(): Promise<string> {
  console.log(`${BLU}${BOLD}\n  ┌──────────────────────────────────────┐`);
  console.log(`  │ TypeScript — Performance & Verbosity │`);
  console.log(`  └──────────────────────────────────────┘${RST}`);
  console.log(`${DIM}  Part 3: memory management cost in code and latency.\n${RST}`);
  console.log(`  ${CYN}1)${RST} Shared ownership — verbosity comparison`);
  console.log(`  ${CYN}2)${RST} Allocation latency — deterministic free vs GC`);
  console.log(`  ${DIM}q) Quit${RST}`);
  process.stdout.write(`\n  ${BOLD}> ${RST}`);
  if (rlClosed) process.exit(0);
  return new Promise(resolve =>
    rl.once("line", line => { console.log(); resolve(line.trim()); })
  );
}

async function main() {
  for (;;) {
    const choice = await printMenu();
    if      (choice === "1") await demoSharedOwnership();
    else if (choice === "2") await demoAllocLatency();
    else if (choice === "q" || choice === "Q") {
      console.log(`${DIM}\n  Exiting. GC will handle cleanup whenever it feels like it.\n${RST}`);
      rl.close();
      break;
    } else console.log(`${RED}  Unknown option.${RST}`);
  }
}

main();
