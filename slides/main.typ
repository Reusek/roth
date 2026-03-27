// ── Palette ──────────────────────────────────
#let C = (
  bg:    rgb("#fafafa"),
  fg:    rgb("#1a1a2e"),
  dim:   rgb("#71717a"),
  acc:   rgb("#e63946"),
  blue:  rgb("#2563eb"),
  green: rgb("#16a34a"),
  amber: rgb("#d97706"),
  dark:  rgb("#1a1a2e"),
  code:  rgb("#1e1e2e"),
)

// ── Page setup ──────────────────────────────
#set page(
  width: 13.333in, height: 7.5in,
  margin: (x: 0.9in, top: 0.7in, bottom: 0.5in),
  fill: C.bg,
)
#set text(size: 20pt, fill: C.fg, font: "New Computer Modern Sans")
#set par(leading: 0.65em)
#show raw: set text(size: 15pt)

// ── Helpers ─────────────────────────────────
#let tag(body, color: C.acc) = box(
  fill: color, radius: 4pt, inset: (x: 10pt, y: 5pt),
)[#text(fill: white, size: 11pt, weight: "bold")[#body]]

#let code-block(body) = block(
  fill: C.code, radius: 8pt, inset: 16pt, width: 100%,
)[#text(fill: rgb("#cdd6f4"))[#body]]

#let slide(title, body) = {
  block(width: 100%, height: 100%)[
    #text(size: 36pt, weight: "bold")[#title]
    #v(20pt)
    #body
  ]
  pagebreak()
}

#let note(body) = text(size: 16pt, fill: C.dim)[#body]

// ═══════════════════════════════════════════════
// SLIDE 1 — Title
// ═══════════════════════════════════════════════

#set page(fill: C.dark)
#block(width: 100%, height: 100%)[
  #v(1fr)
  #tag[BAKALÁŘSKÁ PRÁCE]
  #v(16pt)
  #text(size: 56pt, weight: "bold", fill: white)[Roth]
  #v(8pt)
  #text(size: 24pt, fill: rgb("#a1a1aa"))[
    Zásobníkový jazyk inspirovaný Forthem, implementovaný v Rustu
  ]
  #v(28pt)
  #h(0pt)
  #tag([Kompilátor], color: C.blue)
  #h(8pt)
  #tag([Optimalizace], color: C.green)
  #h(8pt)
  #tag([REPL], color: C.amber)
  #v(1fr)
  #text(size: 18pt, fill: white, weight: "semibold")[Albert Klinkovský]
  #h(12pt)
  #text(size: 15pt, fill: rgb("#a1a1aa"))[Katedra informatiky · jaro 2026]
]
#pagebreak()
#set page(fill: C.bg)

// ═══════════════════════════════════════════════
// SLIDE 2 — Co je Forth
// ═══════════════════════════════════════════════

#slide([Co je Forth?])[
  #grid(
    columns: (1fr, 1fr),
    column-gutter: 48pt,
    [
      Jazyk z roku 1970 postavený na třech myšlenkách:

      #v(12pt)

      + *Zásobník* — data se předávají přes zásobník, ne parametry
      + *Slova* — funkce definované pomocí `: jméno ... ;`
      + *Postfix notace* — `2 3 +` místo `2 + 3`

      #v(16pt)
      #note[Minimalistický, ale překvapivě výkonný — používal se v~embedded systémech, astronomii i~bootloaderech.]
    ],
    [
      #code-block[
        ```forth
        : square  dup * ;
        5 square .          \ => 25
        ```
      ]
      #v(16pt)
      #grid(
        columns: (1fr, 1fr),
        column-gutter: 16pt,
        row-gutter: 10pt,
        [#tag([DUP], color: C.blue) duplikuje vrchol],
        [#tag([SWAP], color: C.amber) prohodí dva prvky],
        [#tag([DROP], color: C.acc) zahodí vrchol],
        [#tag([OVER], color: C.green) kopíruje druhý prvek],
      )
    ],
  )
]

// ═══════════════════════════════════════════════
// SLIDE 3 — Proč Roth
// ═══════════════════════════════════════════════

#slide([Proč Roth?])[
  #grid(
    columns: (1fr, 1fr),
    column-gutter: 48pt,
    [
      === Cíl práce
      Navrhnout a implementovat *kompletní kompilátor* pro zásobníkový jazyk:

      - kompilátor s~více fázemi
      - optimalizační průchody nad IR
      - backendy pro Rust a C
      - interaktivní REPL
    ],
    [
      === Proč Rust?

      - typová bezpečnost bez garbage collectoru
      - `enum` + `match` = přirozený fit pro AST/IR
      - pattern matching pro transformace
      - silný ekosystém (_clap_, _rustyline_, …)
    ],
  )
  #v(1fr)
  #grid(
    columns: (1fr, 1fr, 1fr),
    column-gutter: 24pt,
    [#text(size: 32pt, weight: "bold", fill: C.blue)[~4 500] #note[řádků Rustu]],
    [#text(size: 32pt, weight: "bold", fill: C.amber)[2] #note[backendy (Rust + C)]],
    [#text(size: 32pt, weight: "bold", fill: C.green)[5] #note[fází kompilace]],
  )
]

// ═══════════════════════════════════════════════
// SLIDE 4 — Pipeline
// ═══════════════════════════════════════════════

#slide([Pipeline kompilátoru])[
  #v(8pt)
  #align(center)[
    #grid(
      columns: (1fr, auto, 1fr, auto, 1fr, auto, 1fr, auto, 1fr),
      column-gutter: 0pt,
      align: center + horizon,
      block(fill: rgb("#fee2e2"), radius: 8pt, inset: (x: 12pt, y: 14pt), width: 100%)[
        #text(weight: "bold")[Lexer]
        #v(4pt)
        #text(size: 14pt, fill: C.dim)[kód → tokeny]
      ],
      text(size: 24pt, fill: C.dim)[  →  ],
      block(fill: rgb("#dbeafe"), radius: 8pt, inset: (x: 12pt, y: 14pt), width: 100%)[
        #text(weight: "bold")[Parser]
        #v(4pt)
        #text(size: 14pt, fill: C.dim)[tokeny → AST]
      ],
      text(size: 24pt, fill: C.dim)[  →  ],
      block(fill: rgb("#dcfce7"), radius: 8pt, inset: (x: 12pt, y: 14pt), width: 100%)[
        #text(weight: "bold")[Analýza]
        #v(4pt)
        #text(size: 14pt, fill: C.dim)[sémantika]
      ],
      text(size: 24pt, fill: C.dim)[  →  ],
      block(fill: rgb("#fef3c7"), radius: 8pt, inset: (x: 12pt, y: 14pt), width: 100%)[
        #text(weight: "bold")[IR]
        #v(4pt)
        #text(size: 14pt, fill: C.dim)[mezikód]
      ],
      text(size: 24pt, fill: C.dim)[  →  ],
      block(fill: rgb("#e9d5ff"), radius: 8pt, inset: (x: 12pt, y: 14pt), width: 100%)[
        #text(weight: "bold")[Backend]
        #v(4pt)
        #text(size: 14pt, fill: C.dim)[Rust / C]
      ],
    )
  ]

  #v(1fr)

  #grid(
    columns: (1fr, 1fr),
    column-gutter: 48pt,
    [
      === Modulární architektura
      Každá fáze má jasně definovaný vstup a výstup — chyby se izolují na konkrétní místo.
    ],
    [
      === Rozšiřitelnost
      Nový backend = nový modul. Zbytek pipeline zůstává beze změny.
    ],
  )
]

// ═══════════════════════════════════════════════
// SLIDE 5 — Lexer & Parser
// ═══════════════════════════════════════════════

#slide([Lexer a Parser])[
  #grid(
    columns: (1fr, 1.2fr),
    column-gutter: 48pt,
    [
      === Lexer
      - tokenizace zdrojového kódu
      - čísla, identifikátory, klíčová slova
      - sledování pozic pro chybové hlášky

      #v(16pt)

      === Parser
      - tokeny → AST
      - definice slov (`: název ... ;`)
      - řídicí struktury (`IF`, `ELSE`, `DO`, `LOOP`)
    ],
    [
      #code-block[
        ```text
        Vstup:   : double  2 * ;

        Tokeny:  Colon, Ident("double"),
                 Num(2), Star, Semicolon

        AST:     WordDef {
                   name: "double",
                   body: [Push(2), Mul]
                 }
        ```
      ]
    ],
  )
]

// ═══════════════════════════════════════════════
// SLIDE 6 — IR a codegen
// ═══════════════════════════════════════════════

#slide([IR a generování kódu])[
  #grid(
    columns: (1fr, 1.2fr),
    column-gutter: 48pt,
    [
      === Mezikód (IR)
      - abstrakce nad cílovým jazykem
      - optimalizace nezávisle na backendu
      - lineární sekvence instrukcí

      #v(16pt)

      === Backendy
      - *Rust* — zásobník jako `Vec<i64>`
      - *C* — pole + ukazatel jako zásobník
    ],
    [
      #code-block[
        ```text
        IR:   Push(5), Push(3), Add, Print

        Rust: stack.push(5);
              stack.push(3);
              let b = stack.pop().unwrap();
              let a = stack.pop().unwrap();
              stack.push(a + b);

        C:    stack[sp++] = 5;
              stack[sp++] = 3;
              sp--; stack[sp-1] += stack[sp];
        ```
      ]
    ],
  )
]

// ═══════════════════════════════════════════════
// SLIDE 7 — Optimalizace
// ═══════════════════════════════════════════════

#slide([Optimalizace])[
  #grid(
    columns: (1fr, 1fr),
    column-gutter: 48pt,
    [
      Pět průchodů nad IR:

      #v(8pt)

      + *Constant folding* — `2 3 +` → `5` v compile time
      + *Dead code elimination* — odstranění nedosažitelného kódu
      + *Peephole* — `DUP DROP` → nic
      + *Strength reduction* — `2 *` → bitový posun
      + *Inlining* — krátká slova přímo na místo volání
    ],
    [
      #code-block[
        ```text
        Před:
          Push(2), Push(3), Add,
          Push(1), Mul,
          Dup, Drop

        Po:
          Push(5)
        ```
      ]

      #v(12pt)
      #note[Tři průchody, tři eliminace — výsledek je jediná instrukce.]
    ],
  )
]

// ═══════════════════════════════════════════════
// SLIDE 8 — REPL
// ═══════════════════════════════════════════════

#slide([REPL])[
  #grid(
    columns: (1fr, 1.2fr),
    column-gutter: 48pt,
    [
      === Funkce
      - okamžité vyhodnocení výrazů
      - definice slov za běhu
      - zobrazení stavu zásobníku
      - historie příkazů

      #v(16pt)

      === Implementace
      - `rustyline` pro editaci příkazové řádky
      - JIT přístup — kompilace na vyžádání
      - runtime knihovna se standardními slovy
    ],
    [
      #code-block[
        ```text
        roth> 5 3 +
        [8]

        roth> : fact
          dup 1 > if dup 1 - fact * then ;
        OK

        roth> 10 fact .
        3628800

        roth> .s
        Stack: []
        ```
      ]
    ],
  )
]

// ═══════════════════════════════════════════════
// SLIDE 9 — Testování
// ═══════════════════════════════════════════════

#slide([Testování])[
  #grid(
    columns: (1fr, 1fr),
    column-gutter: 48pt,
    [
      === Jednotkové testy
      Každá fáze kompilátoru má vlastní sadu testů — izolovaně ověřují správnost transformací.

      #v(16pt)
      Testované komponenty:

      #v(8pt)
      #grid(
        columns: (auto, auto, auto, auto, auto),
        column-gutter: 8pt,
        tag([Lexer], color: C.acc),
        tag([Parser], color: C.blue),
        tag([IR], color: C.amber),
        tag([Codegen], color: C.green),
        tag([Chyby], color: rgb("#7c3aed")),
      )
    ],
    [
      === Integrační testy
      Celé programy v~Rothu se kompilují a spouští — výstup se porovnává s~očekávanou hodnotou.

      #v(16pt)
      === End-to-end
      Ověření celé pipeline od zdrojového kódu po spustitelný výstup obou backendů.
    ],
  )
]

// ═══════════════════════════════════════════════
// SLIDE 10 — Výsledky
// ═══════════════════════════════════════════════

#slide([Výsledky])[
  #grid(
    columns: (1fr, 1fr),
    column-gutter: 48pt,
    [
      === Co vzniklo
      - kompletní zásobníkový jazyk Roth
      - kompilátor s~5 fázemi pipeline
      - 5 optimalizačních průchodů nad IR
      - dva backendy — Rust a C
      - interaktivní REPL s~runtime knihovnou
    ],
    [
      === Přínos
      - funkční nástroj pro experimentování se zásobníkovými jazyky
      - modulární architektura — snadné rozšíření
      - zdokumentovaný návrh a implementace
    ],
  )

  #v(1fr)

  #grid(
    columns: (1fr, 1fr, 1fr, 1fr),
    column-gutter: 24pt,
    [#text(size: 36pt, weight: "bold", fill: C.blue)[~4 500] #note[řádků Rustu]],
    [#text(size: 36pt, weight: "bold", fill: C.amber)[5] #note[fází pipeline]],
    [#text(size: 36pt, weight: "bold", fill: C.green)[2] #note[backendy]],
    [#text(size: 36pt, weight: "bold", fill: C.acc)[5] #note[optim. průchodů]],
  )
]

// ═══════════════════════════════════════════════
// SLIDE 11 — Děkuji
// ═══════════════════════════════════════════════

#set page(fill: C.dark)
#block(width: 100%, height: 100%)[
  #v(1fr)
  #text(size: 52pt, weight: "bold", fill: white)[Děkuji za pozornost]
  #v(12pt)
  #text(size: 24pt, fill: rgb("#a1a1aa"))[Dotazy?]
  #v(1fr)
  #tag([Albert Klinkovský], color: C.blue)
  #h(8pt)
  #tag([Katedra informatiky], color: C.green)
  #h(8pt)
  #tag([Jaro 2026], color: C.amber)
]
