import * as readline from "readline";

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

const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
let rlClosed = false;
rl.on("close", () => { rlClosed = true; });

function ask(question: string): Promise<string> {
  return new Promise((resolve) => {
    if (rlClosed) { resolve("q"); return; }
    const onClose = () => resolve("q");
    rl.once("close", onClose);
    rl.question(question, (answer) => {
      rl.removeListener("close", onClose);
      resolve(answer);
    });
  });
}

const CL = `  ${DIM}│${RST}  `;

function codeOpen(lang: string, color: string): void {
  console.log(`${DIM}  ╭─ ${RST}${BOLD}${color}${lang}${RST} ${DIM}────────────────────────────────────────────────────────`);
  console.log(`  │${RST}`);
}
const codeOpenC  = () => codeOpen("c",  RED);
const codeOpenTs = () => codeOpen("ts", BLU);

function codeClose(): void {
  console.log(`${DIM}  │`);
  console.log(`  ╰──────────────────────────────────────────────────────────${RST}`);
  console.log();
}

function sectionResult(): void {
  console.log(`${BOLD}${CYN}\n  ━━ Result ${RST}${DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RST}\n`);
}

function sectionExplain(): void {
  console.log(`${BOLD}${MAG}\n  ━━ What happened ${RST}${DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RST}\n`);
}

async function pressEnter(question: string): Promise<void> {
  console.log(`${BOLD}${YLW}\n  ╔═ ? ════════════════════════════════════════════════════════╗${RST}`);
  console.log(`${BOLD}${YLW}  ║${RST}  ${question}`);
  console.log(`${BOLD}${YLW}  ║${RST}`);
  console.log(`${BOLD}${YLW}  ║${RST}  ${DIM}Press Enter to see what actually happens...${RST}`);
  console.log(`${BOLD}${YLW}  ╚════════════════════════════════════════════════════════════╝${RST}`);
  await ask("");
  console.log();
}

// ============================================================
// Demo 1: GC prevents dangling — references keep objects alive
// ============================================================
async function demoNoDangling(): Promise<void> {
  console.log(`  In C, returning a pointer to a local variable is dangling UB:\n`);

  codeOpenC();
  console.log(`${CL}${YLW}int${RST} *bad_get() {`);
  console.log(`${CL}    ${YLW}int${RST} local = ${MAG}42${RST};`);
  console.log(`${CL}    ${RED}return &local;${RST}  ${GRY}// UB: stack frame gone on return${RST}`);
  console.log(`${CL}}`);
  codeClose();

  console.log(`  In TypeScript, returning an object reference is always safe:\n`);

  codeOpenTs();
  console.log(`${CL}${YLW}function${RST} getRef(): ${GRN}{ value: number }${RST} {`);
  console.log(`${CL}    ${YLW}const${RST} obj = { value: ${MAG}42${RST} };`);
  console.log(`${CL}    ${YLW}return${RST} obj;  ${GRY}// passes reference — GC sees caller holds it${RST}`);
  console.log(`${CL}}`);
  console.log(`${DIM}  │${RST}`);
  console.log(`${CL}${YLW}const${RST} ref1 = getRef();`);
  console.log(`${CL}${YLW}const${RST} ref2 = getRef();`);
  console.log(`${CL}console.log(ref1.value, ref2.value);  ${GRY}// always valid${RST}`);
  codeClose();

  await pressEnter("No stack to destroy — what lifetime does a JS object have?");
  sectionResult();

  function getRef() { return { value: 42 }; }

  const ref1 = getRef();
  const ref2 = getRef();
  console.log(`  ref1.value = ${GRN}${BOLD}${ref1.value}${RST}  (alive: ref1 holds it)`);
  console.log(`  ref2.value = ${GRN}${BOLD}${ref2.value}${RST}  (alive: ref2 holds it)\n`);

  // Demonstrate multiple references to same object
  const obj = { value: 99 };
  const a = obj;
  const b = obj;
  a.value = 100;
  console.log(`  Shared reference: a and b both point to the same object`);
  console.log(`  a.value = ${GRN}${BOLD}${a.value}${RST}`);
  console.log(`  b.value = ${GRN}${BOLD}${b.value}${RST}  (same object, not a copy)`);

  sectionExplain();
  console.log(`  JavaScript objects live on the ${BOLD}heap${RST}, managed by a garbage collector.`);
  console.log(`  An object is kept alive as long as ${BOLD}any reference to it exists${RST}.\n`);
  console.log(`  When getRef() returns, the local variable ${GRN}obj${RST} is gone — but`);
  console.log(`  the ${BOLD}reference${RST} was returned to the caller, so the GC sees the`);
  console.log(`  object is still reachable. It will not be collected.\n`);
  console.log(`  There is ${BOLD}no stack${RST} in the C sense. Objects are not tied to`);
  console.log(`  function call frames. The concept of a dangling pointer ${GRN}does not exist${RST}.`);
}

// ============================================================
// Demo 2: No use-after-free — GC owns the free timing
// ============================================================
async function demoNoUseAfterFree(): Promise<void> {
  console.log(`  In C, free() returns memory to the allocator immediately:\n`);

  codeOpenC();
  console.log(`${CL}${YLW}int${RST} *p = ${CYN}malloc${RST}(${YLW}sizeof${RST}(${YLW}int${RST}));`);
  console.log(`${CL}*p = ${MAG}42${RST};`);
  console.log(`${CL}${CYN}free${RST}(p);           ${GRY}// heap freed now${RST}`);
  console.log(`${CL}${YLW}int${RST} x = *p;        ${RED}// ← use-after-free: UB${RST}`);
  codeClose();

  console.log(`  In TypeScript, you cannot free memory — the GC decides when:\n`);

  codeOpenTs();
  console.log(`${CL}${YLW}let${RST} owned: ${GRN}{ value: number }${RST} | ${RED}null${RST} = { value: ${MAG}42${RST} };`);
  console.log(`${CL}${YLW}const${RST} shared = owned;    ${GRY}// second reference${RST}`);
  console.log(`${DIM}  │${RST}`);
  console.log(`${CL}owned = ${RED}null${RST};           ${GRY}// drop our reference${RST}`);
  console.log(`${CL}${GRY}// GC will NOT collect: 'shared' still holds a reference${RST}`);
  console.log(`${CL}console.log(shared.value); ${GRN}// always safe${RST}`);
  codeClose();

  await pressEnter("Setting a variable to null — does that free the object? What if another variable holds it?");
  sectionResult();

  let owned: { value: number } | null = { value: 42 };
  const shared = owned;

  console.log(`  Before: owned.value = ${GRN}${BOLD}${owned.value}${RST}, shared.value = ${GRN}${BOLD}${shared.value}${RST}`);

  owned = null;
  console.log(`  After owned = null:`);
  console.log(`    owned  = ${YLW}${owned}${RST}`);
  console.log(`    shared.value = ${GRN}${BOLD}${shared.value}${RST}  (object still alive via shared)`);

  // Show that WeakRef loses access after GC
  console.log();
  console.log(`  WeakRef — holds a reference the GC can ignore:\n`);
  let strong: object | null = { secret: "still here" };
  const weak = new WeakRef(strong);
  console.log(`  weak.deref() before release = ${GRN}${BOLD}${JSON.stringify(weak.deref())}${RST}`);
  strong = null;
  console.log(`  strong = null (only weak reference remains)`);
  console.log(`  ${DIM}(GC may collect this — deref() could return undefined after GC runs)${RST}`);
  console.log(`  weak.deref() right now      = ${YLW}${BOLD}${JSON.stringify(weak.deref())}${RST}  ${DIM}(not yet collected)${RST}`);

  sectionExplain();
  console.log(`  ${BOLD}You cannot free memory in JavaScript${RST}. Setting a variable to ${RED}null${RST}`);
  console.log(`  just removes ${BOLD}your${RST} reference. The GC tracks all references and`);
  console.log(`  only frees the object when ${BOLD}none remain${RST}.\n`);
  console.log(`  ${GRN}WeakRef${RST} is the only way to hold a reference the GC can ignore.`);
  console.log(`  After the strong reference is gone, ${YLW}weak.deref()${RST} may return`);
  console.log(`  ${RED}undefined${RST} at any point after the next GC cycle.\n`);
  console.log(`  The tradeoff: you cannot ${BOLD}control when${RST} memory is freed.`);
  console.log(`  For most apps this is fine. For tight memory budgets (games,`);
  console.log(`  embedded, high-throughput servers) GC pauses can be a problem.`);
  console.log(`  This is one reason Rust exists — ${GRN}deterministic deallocation${RST}`);
  console.log(`  without a GC.`);
}

// ============================================================
// Demo 3: No double-free — the concept does not exist
// ============================================================
async function demoNoDoubleFree(): Promise<void> {
  console.log(`  In C, you can call free() on the same pointer twice:\n`);

  codeOpenC();
  console.log(`${CL}${YLW}int${RST} *p = ${CYN}malloc${RST}(${YLW}sizeof${RST}(${YLW}int${RST}));`);
  console.log(`${CL}${CYN}free${RST}(p);  ${GRY}// correct${RST}`);
  console.log(`${CL}${CYN}free${RST}(p);  ${RED}// double-free: heap corruption / SIGABRT${RST}`);
  codeClose();

  console.log(`  In TypeScript, there is no free() — GC owns all deallocation:\n`);

  codeOpenTs();
  console.log(`${CL}${YLW}let${RST} a: ${GRN}object${RST} | ${RED}null${RST} = { value: ${MAG}42${RST} };`);
  console.log(`${CL}${YLW}let${RST} b = a;        ${GRY}// second reference to same object${RST}`);
  console.log(`${DIM}  │${RST}`);
  console.log(`${CL}a = ${RED}null${RST};          ${GRY}// remove one reference${RST}`);
  console.log(`${CL}b = ${RED}null${RST};          ${GRY}// remove last reference${RST}`);
  console.log(`${CL}${GRY}// GC now eligible to collect — no double-free possible${RST}`);
  console.log(`${DIM}  │${RST}`);
  console.log(`${CL}a = ${RED}null${RST};          ${GRY}// nulling null: perfectly fine${RST}`);
  console.log(`${CL}b = ${RED}null${RST};          ${GRY}// same — no-op, no error${RST}`);
  codeClose();

  await pressEnter("There's no free(). How does memory ever actually get reclaimed?");
  sectionResult();

  let a: { value: number; refs?: string } | null = { value: 42, refs: "a,b" };
  let b: typeof a = a;

  console.log(`  [a and b point to same object]  a===b: ${GRN}${BOLD}${a === b}${RST}`);

  a = null;
  console.log(`  After a=null: b.value = ${GRN}${BOLD}${b?.value}${RST}  (still alive via b)`);

  b = null;
  console.log(`  After b=null: no references remain`);
  console.log(`  ${DIM}GC will collect the object at its next cycle — we have no`);
  console.log(`  handle, no address, no way to access or double-free it${RST}\n`);

  console.log(`  Attempting to "double-null" is a harmless no-op:`);
  a = null;
  b = null;
  console.log(`  a = null again: ${GRN}${BOLD}ok${RST}  (a = ${a}, b = ${b})`);

  sectionExplain();
  console.log(`  The GC maintains a ${BOLD}reference count${RST} (or reachability graph).`);
  console.log(`  An object is freed exactly ${BOLD}once${RST}: when the count reaches zero.\n`);
  console.log(`  You cannot trigger a double-free because:`);
  console.log(`    ${GRN}•${RST} There is no ${RED}free()${RST} to call`);
  console.log(`    ${GRN}•${RST} Setting a variable to ${RED}null${RST} just decrements the count`);
  console.log(`    ${GRN}•${RST} The GC calls the internal free exactly once\n`);
  console.log(`  The cost: ${YLW}non-deterministic timing${RST}. You cannot predict exactly`);
  console.log(`  when memory is reclaimed. For resource-constrained systems`);
  console.log(`  or real-time requirements, this is why manual/ownership-based`);
  console.log(`  memory management (C, Rust) is still used.`);
}

// ============================================================
// Demo 4: undefined as null — optional chaining / null safety
// ============================================================
function findValue(arr: number[], target: number): number | undefined {
  return arr.find(x => x === target);
}

async function demoNullSafety(): Promise<void> {
  console.log(`  In C, find_value returns NULL — type is just int *, you must check:\n`);

  codeOpenC();
  console.log(`${CL}${YLW}int${RST} *p = find_value(arr, n, ${RED}99${RST}); ${GRY}// might be NULL${RST}`);
  console.log(`${CL}${GRY}// C type: int * — same whether valid or NULL${RST}`);
  console.log(`${CL}${CYN}printf${RST}(${GRN}"%%d\\n"${RST}, *p);  ${RED}// crash if NULL — nothing warned you${RST}`);
  codeClose();

  console.log(`  In TypeScript the return type encodes the absent case:\n`);

  codeOpenTs();
  console.log(`${CL}${YLW}function${RST} findValue(arr: ${GRN}number[]${RST}, t: ${YLW}number${RST}): ${GRN}number | undefined${RST} {`);
  console.log(`${CL}    ${YLW}return${RST} arr.${CYN}find${RST}(x => x === t);`);
  console.log(`${CL}}`);
  console.log(`${DIM}  │${RST}`);
  console.log(`${CL}${YLW}const${RST} v = findValue(arr, ${RED}99${RST});`);
  console.log(`${CL}v${RED}.toFixed(2)${RST};     ${RED}// TS error: v is number | undefined${RST}`);
  console.log(`${CL}v${GRN}?.toFixed(2)${RST};    ${GRN}// safe: optional chain, returns undefined${RST}`);
  console.log(`${CL}(v ${GRN}?? 0${RST}).toFixed(${MAG}2${RST}); ${GRN}// safe: fallback to 0${RST}`);
  codeClose();

  await pressEnter("TypeScript knows findValue might return undefined. What does that mean at runtime?");
  sectionResult();

  const arr = [1, 2, 3, 4, 5];

  for (const target of [3, 99]) {
    const v = findValue(arr, target);
    if (v !== undefined) {
      console.log(`  findValue(arr, ${String(target).padStart(2)}) = ${GRN}${BOLD}${v}${RST}  (found)`);
    } else {
      console.log(`  findValue(arr, ${String(target).padStart(2)}) = ${RED}${BOLD}undefined${RST}  (not found)`);
    }
  }

  console.log();
  const v = findValue(arr, 99);  // undefined
  console.log(`  v?.toFixed(2)       = ${YLW}${BOLD}${v?.toFixed(2)}${RST}   (optional chain short-circuits)`);
  console.log(`  (v ?? 0).toFixed(2) = ${GRN}${BOLD}${(v ?? 0).toFixed(2)}${RST}  (nullish coalescing provides default)`);
  console.log(`  v ?? "not found"    = ${GRN}${BOLD}"${v ?? "not found"}"${RST}`);

  sectionExplain();
  console.log(`  TypeScript's ${RED}null${RST} / ${RED}undefined${RST} in a union type makes the absent`);
  console.log(`  case ${BOLD}visible in the type system${RST}. The compiler rejects code that`);
  console.log(`  calls methods on ${GRN}number | undefined${RST} without narrowing first.\n`);
  console.log(`  Three patterns for safe access:`);
  console.log(`    ${GRN}v?.method()${RST}    — optional chaining: returns undefined if v is nil`);
  console.log(`    ${GRN}v ?? fallback${RST}  — nullish coalescing: substitute a default`);
  console.log(`    ${GRN}if (v !== undefined)${RST} — type narrowing: v is ${YLW}number${RST} inside\n`);
  console.log(`  Enable ${YLW}strictNullChecks${RST} in tsconfig (included in ${YLW}strict${RST}) to get`);
  console.log(`  this protection. Without it, ${RED}null${RST} and ${RED}undefined${RST} are assignable`);
  console.log(`  to every type — the TS equivalent of C's unguarded NULL pointer.`);
}

function printMenu(): void {
  console.log(`${BLU}${BOLD}\n  ┌──────────────────────────────────────┐`);
  console.log(`  │    TypeScript / GC Memory Safety     │`);
  console.log(`  └──────────────────────────────────────┘${RST}`);
  console.log(`${DIM}  Part 2: GC lifetimes, no free(), WeakRef, null safety.\n${RST}`);
  console.log(`  ${CYN}1)${RST} No dangling — GC keeps objects alive`);
  console.log(`  ${CYN}2)${RST} No use-after-free — GC owns the free timing`);
  console.log(`  ${CYN}3)${RST} No double-free — concept doesn't exist`);
  console.log(`  ${CYN}4)${RST} Null safety — undefined in the type system`);
  console.log(`  ${DIM}q) Quit${RST}`);
}

async function main(): Promise<void> {
  while (true) {
    printMenu();
    const choice = await ask(`\n  ${BOLD}> ${RST}`);
    console.log();

    switch (choice.trim()) {
      case "1": await demoNoDangling();    break;
      case "2": await demoNoUseAfterFree(); break;
      case "3": await demoNoDoubleFree();  break;
      case "4": await demoNullSafety();    break;
      case "q": case "Q":
        console.log(`${DIM}\n  Exiting. Memory reclaimed whenever the GC gets around to it.\n${RST}`);
        rl.close();
        return;
      default:
        console.log(`${RED}  Unknown option.${RST}`);
    }
  }
}

main();
