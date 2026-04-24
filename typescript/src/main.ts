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

const CL = `  ${DIM}│${RST}  `;  // code-line prefix

function codeOpen(): void {
  console.log(`${DIM}  ╭─ ${RST}${BOLD}${BLU}ts${RST} ${DIM}────────────────────────────────────────────────────────`);
  console.log(`  │${RST}`);
}

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
// Demo 1: Arrays always carry length
// ============================================================
async function demoLengthAlwaysPresent(): Promise<void> {
  codeOpen();
  console.log(`${CL}${DIM}// arrays are heap objects — length is always a property${RST}`);
  console.log(`${CL}${YLW}function${RST} printLength(arr: ${GRN}number[]${RST}): ${YLW}void${RST} {`);
  console.log(`${CL}    console.log(arr.${CYN}length${RST});`);
  console.log(`${CL}}`);
  console.log(`${DIM}  │${RST}`);
  console.log(`${CL}${YLW}const${RST} data = [${MAG}1${RST}, ${MAG}2${RST}, ${MAG}3${RST}, ${MAG}4${RST}, ${MAG}5${RST}, ${MAG}6${RST}, ${MAG}7${RST}, ${MAG}8${RST}, ${MAG}9${RST}, ${MAG}10${RST}];`);
  console.log(`${CL}console.log(data.${CYN}length${RST});  ${GRY}// ← in calling scope${RST}`);
  console.log(`${CL}printLength(data);          ${GRY}// ← inside function${RST}`);
  codeClose();

  await pressEnter("In C, length is lost when passing to functions. What does TypeScript do?");
  sectionResult();

  const data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
  console.log(`  data.length in calling scope  : ${GRN}${BOLD}${data.length}${RST}`);
  printLength(data);

  const partial = data.slice(2, 5);
  console.log(`  data.slice(2,5).length        : ${GRN}${BOLD}${partial.length}${RST}`);
  printLength(partial);

  sectionExplain();
  console.log(`  Arrays in JS/TS are ${BOLD}heap-allocated objects${RST}.`);
  console.log(`  ${GRN}.length${RST} is always a property of the object — it ${BOLD}cannot be`);
  console.log(`  separated from it${RST}. Passing an array passes a reference to`);
  console.log(`  the same object. The function always has full access to .length.\n`);
  console.log(`  There is ${BOLD}no decay${RST}. The concept does not exist at this level.`);
}

function printLength(arr: number[]): void {
  console.log(`  arr.length inside function    : ${GRN}${BOLD}${arr.length}${RST}`);
}

// ============================================================
// Demo 2: Out-of-bounds read returns undefined -- no crash
// ============================================================
async function demoOutOfBounds(): Promise<void> {
  codeOpen();
  console.log(`${CL}${YLW}const${RST} arr = [${MAG}10${RST}, ${MAG}20${RST}, ${MAG}30${RST}];`);
  console.log(`${CL}console.log(arr[${GRN}3${RST}]);      ${GRY}// one past the end${RST}`);
  console.log(`${CL}console.log(arr[${RED}99${RST}]);     ${GRY}// way past the end${RST}`);
  console.log(`${CL}console.log(arr[${RED}3${RST}]! + ${MAG}1${RST}); ${GRY}// arithmetic on the result${RST}`);
  codeClose();

  await pressEnter("What do you think arr[3] returns? A crash? An error? A number?");
  sectionResult();

  const arr = [10, 20, 30];
  console.log(`  arr[0]       = ${GRN}${BOLD}${arr[0]!}${RST}          (valid)`);
  console.log(`  arr[1]       = ${GRN}${BOLD}${arr[1]!}${RST}          (valid)`);
  console.log(`  arr[2]       = ${GRN}${BOLD}${arr[2]!}${RST}          (valid)`);
  console.log(`  arr[3]       = ${RED}${BOLD}${arr[3]}${RST}    (out of bounds)`);
  console.log(`  arr[99]      = ${RED}${BOLD}${arr[99]}${RST}  (out of bounds)`);
  console.log(`  arr[3]! + 1  = ${RED}${BOLD}${arr[3]! + 1}${RST}      (undefined + 1 → NaN)`);

  sectionExplain();
  console.log(`  ${GRN}No segfault. No memory corruption.${RST} Arrays are objects, not`);
  console.log(`  raw memory — reading a missing key returns ${RED}undefined${RST}.\n`);
  console.log(`  The silent risk: ${BOLD}undefined propagates through arithmetic as NaN${RST}.`);
  console.log(`  NaN is not an error — it ${RED}silently poisons every calculation${RST}`);
  console.log(`  it touches. You may not notice until the result is displayed.\n`);
  console.log(`  Better than C (no corruption), but still a ${YLW}silent failure mode${RST}.`);
}

// ============================================================
// Demo 3: Out-of-bounds write -- no adjacent memory corruption
// ============================================================
async function demoOutOfBoundsWrite(): Promise<void> {
  codeOpen();
  console.log(`${CL}${YLW}const${RST} arr = [${MAG}1${RST}, ${MAG}2${RST}, ${MAG}3${RST}];`);
  console.log(`${CL}${YLW}const${RST} neighbour = { value: ${GRN}0xDEAD${RST} };`);
  console.log(`${CL}arr[${RED}5${RST}] = ${RED}0xBEEF${RST};  ${GRY}// past the end${RST}`);
  codeClose();

  console.log(`  ${DIM}In C, writing past an array corrupts adjacent memory.${RST}`);

  await pressEnter("What does JS/TS do when you write past the end of an array?");
  sectionResult();

  const arr = [1, 2, 3];
  const neighbour = { value: 0xdead };

  console.log(`  Before: arr=[${arr}]  neighbour.value=${GRN}0x${neighbour.value.toString(16).toUpperCase()}${RST}`);
  (arr as number[])[5] = 0xbeef;
  console.log(`  After arr[5]=0xBEEF:`);
  console.log(`    arr         = [${arr}]  ${DIM}(sparse — holes are undefined)${RST}`);
  console.log(`    arr.length  = ${YLW}${BOLD}${arr.length}${RST}  (extended to cover index 5)`);
  console.log(`    arr[3]      = ${RED}${arr[3]}${RST}  (hole)`);
  console.log(`    arr[4]      = ${RED}${arr[4]}${RST}  (hole)`);
  console.log(`    arr[5]      = ${GRN}${BOLD}0x${arr[5]!.toString(16).toUpperCase()}${RST}`);
  console.log(`    neighbour   = ${GRN}${BOLD}0x${neighbour.value.toString(16).toUpperCase()}${RST}  (completely untouched)`);

  sectionExplain();
  console.log(`  The array grew. Arrays are ${BOLD}dynamic objects${RST}, not fixed memory.`);
  console.log(`  neighbour is a completely separate heap object — there is ${BOLD}no`);
  console.log(`  way to corrupt it${RST} by indexing another array.\n`);
  console.log(`  ${YLW}Sparse arrays${RST} (holes) have subtle iteration behavior:`);
  console.log(`    ${GRY}arr.forEach(...)  skips holes${RST}`);
  console.log(`    ${GRY}arr.reduce(...)   skips holes${RST}`);
  console.log(`    ${GRY}[...arr]          fills holes with undefined${RST}`);
  console.log(`  No corruption, but ${YLW}unexpected behavior${RST} if you forget holes exist.`);
}

// ============================================================
// Demo 4: noUncheckedIndexedAccess -- the type system helps
// ============================================================
async function demoTypeSafety(): Promise<void> {
  codeOpen();
  console.log(`${CL}${DIM}// tsconfig: { "noUncheckedIndexedAccess": true }${RST}`);
  console.log(`${DIM}  │${RST}`);
  console.log(`${CL}${YLW}const${RST} arr: ${GRN}number[]${RST} = [${MAG}10${RST}, ${MAG}20${RST}, ${MAG}30${RST}];`);
  console.log(`${DIM}  │${RST}`);
  console.log(`${CL}${YLW}const${RST} val = arr[${RED}5${RST}];        ${GRY}// type: number | undefined${RST}`);
  console.log(`${CL}val.${CYN}toFixed${RST}(${MAG}2${RST});            ${RED}// Error TS2532: possibly undefined${RST}`);
  console.log(`${CL}${YLW}const${RST} safe = arr[${RED}5${RST}] ?? ${MAG}0${RST};  ${GRY}// type: number ✓${RST}`);
  codeClose();

  await pressEnter("What type does TypeScript infer for arr[5] with noUncheckedIndexedAccess?");
  sectionResult();

  const arr = [10, 20, 30];

  const val = arr[5];
  console.log(`  arr[5]             = ${RED}${BOLD}${val}${RST}  ${DIM}(type: number | undefined)${RST}`);

  const safe = arr[5] ?? 0;
  console.log(`  arr[5] ?? 0        = ${GRN}${BOLD}${safe}${RST}  ${DIM}(type: number)${RST}`);

  const byAt = arr.at(99);
  console.log(`  arr.at(99)         = ${RED}${BOLD}${byAt}${RST}  ${DIM}(type: number | undefined)${RST}`);

  sectionExplain();
  console.log(`  arr[5] has type ${RED}number | undefined${RST} at compile time.`);
  console.log(`  The compiler ${BOLD}refuses${RST} to let you call .toFixed() on it directly,`);
  console.log(`  because that method does not exist on ${RED}undefined${RST}.\n`);
  console.log(`  The ${GRN}??${RST} operator forces you to provide a fallback — after that`);
  console.log(`  the type is ${GRN}number${RST} and you can use it freely.\n`);
  console.log(`  This opt-in flag makes out-of-bounds access visible in the ${BOLD}type`);
  console.log(`  system${RST}, eliminating silent undefined propagation (the NaN trap).`);
}

// ============================================================

function printMenu(): void {
  console.log(`${BLU}${BOLD}\n  ┌──────────────────────────────────────┐`);
  console.log(`  │      TypeScript Array Safety         │`);
  console.log(`  └──────────────────────────────────────┘${RST}`);
  console.log(`${DIM}  Demonstrates how memory-managed languages handle arrays.\n${RST}`);
  console.log(`  ${CYN}1)${RST} Length always carried`);
  console.log(`  ${CYN}2)${RST} Out-of-bounds read → undefined`);
  console.log(`  ${CYN}3)${RST} Out-of-bounds write → no corruption`);
  console.log(`  ${CYN}4)${RST} noUncheckedIndexedAccess types`);
  console.log(`  ${DIM}q) Quit${RST}`);
}

async function main(): Promise<void> {
  while (true) {
    printMenu();
    const choice = await ask(`\n  ${BOLD}> ${RST}`);
    console.log();

    switch (choice.trim()) {
      case "1": await demoLengthAlwaysPresent(); break;
      case "2": await demoOutOfBounds();         break;
      case "3": await demoOutOfBoundsWrite();    break;
      case "4": await demoTypeSafety();          break;
      case "q": case "Q":
        console.log(`${DIM}\n  Exiting. No undefined behavior was possible.\n${RST}`);
        rl.close();
        return;
      default:
        console.log(`${RED}  Unknown option.${RST}`);
    }
  }
}

main();
