<div align="center">

# Persona Forge.skill

> *"Personify anything — famous minds, or unique characters you create."*

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Claude Code](https://img.shields.io/badge/Claude%20Code-Skill-blueviolet)](https://claude.ai/code)
[![Skills](https://img.shields.io/badge/skills.sh-Compatible-green)](https://skills.sh)

<br>

**Persona Forge is a persona engine: distill the thinking of famous minds, or synthesize custom characters to your specifications.**

<br>

[Examples](#examples) · [Install](#install) · [What It Does](#what-it-does) · [How It Works](#how-it-works)

<br>

**Other Languages:**
[中文](README.md)

</div>

---

## Examples

### Example 1: Distilling a Famous Mind

```
User    ❯ I want to do content creation, write a book, and build an indie app,
           but I never have enough energy.

Naval   ❯ You've listed three desires. Each desire is a contract you signed
           with unhappiness. This isn't about energy — it's about too many contracts.

           Ask yourself: which one makes you lose track of time?
           That's where your specific knowledge lives.

           Not choose one forever. Just one first, then one, then one.
           Serial compounding, not parallel exhaustion.
```

```
User    ❯ Our SaaS customer acquisition cost is too high.

Musk    ❯ Don't think about how to reduce it yet. Calculate the physical minimum first.
           What's the minimum necessary action to acquire a customer?
           What's the theoretically shortest path from knowing you to paying you?

           How many times longer is your actual path vs. the theoretical one?
           If it's more than 3x, there are steps you can eliminate.
           Don't optimize the funnel — question whether the funnel should exist at all.
```

### Example 2: Synthesizing a Custom Character

```
User          ❯ Create a patient elementary school math teacher persona

Persona Forge ❯ Generated "Patient Math Teacher" persona.

User          ❯ Why doesn't 1/2 plus 1/3 equal 2/5?

Math Teacher  ❯ That's a really interesting thought! A lot of kids think that at first.
                  Let's do an experiment: imagine you have a pizza...

                  (Character designed from "warm encouragement + analogy-rich + guided discovery" dimensions)
```

This is not role-play. Naval uses his "desire as contract" mental model. Musk uses "asymptotic limit" reasoning. The math teacher uses "scaffolding + analogy-based pedagogy". **They are not reciting templates — they are analyzing your problem through specific cognitive frameworks.**

---

## Install

```bash
npx skills add alchaincyf/persona-forge
```

Then in Claude Code:

**Distill a famous mind:**
```
> Distill Paul Graham
> Build a Steve Jobs perspective skill
> Create a Naval skill for me
```

**Synthesize a custom character:**
```
> Create a patient math teacher persona
> Build a strict but supportive mentor role
> Design an AI companion for teaching kids to code
```

After creation, invoke directly:
```
> Use Munger's perspective to analyze this investment decision
> How would Feynman explain quantum computing?
> Switch to math teacher, explain fractions to me
```

---

## What It Does

Persona Forge extracts five layers of cognitive structure:

| Layer | Description |
|---|---|
| **How they speak** | Expression DNA — tone, rhythm, word preferences |
| **How they think** | Mental models, cognitive frameworks |
| **How they judge** | Decision heuristics |
| **What they won't do** | Anti-patterns, value floor |
| **Honest limits** | What the skill genuinely cannot do |

### Two Capabilities

**1. Person Distillation** — Input a name, and 6 parallel agents research the person's writings, interviews, and decision records to extract their unique cognitive operating system.

**2. Persona Synthesis** — No specific person needed. Select/combine dimensions from the Persona Dimension Library (teaching style, communication warmth, expression mode, etc.) and generate a custom character in 5 minutes.

### Honest Limits

Every skill explicitly states what it cannot do:

- Cannot distill intuition — frameworks can be extracted, inspiration cannot
- Cannot capture change — only a snapshot up to the research cutoff
- Public statements ≠ true beliefs — only based on public information
- Synthetic personas ≠ real people — clearly labeled as AI-generated characters

**A skill that doesn't tell you its limits is not worth trusting.**

---

## Generated Skills

Persona Forge has generated 13 person skills + 1 topic skill. Each is a standalone, installable skill:

### Person Skills (Distilled)

| Person | Domain | Standalone Repo | Install |
|------|------|---------|---------|
| 🔥 **Paul Graham** | Startups/Writing/Product/Philosophy | [paul-graham-skill](https://github.com/alchaincyf/paul-graham-skill) | `npx skills add alchaincyf/paul-graham-skill` |
| 🔥 **Zhang Yiming** | Product/Organization/Globalization | [zhang-yiming-skill](https://github.com/alchaincyf/zhang-yiming-skill) | `npx skills add alchaincyf/zhang-yiming-skill` |
| 🔥 **Karpathy** | AI/Engineering/Education/Open Source | [karpathy-skill](https://github.com/alchaincyf/karpathy-skill) | `npx skills add alchaincyf/karpathy-skill` |
| 🔥 **Ilya Sutskever** | AI Safety/Scaling/Research Taste | [ilya-sutskever-skill](https://github.com/alchaincyf/ilya-sutskever-skill) | `npx skills add alchaincyf/ilya-sutskever-skill` |
| 🔥 **MrBeast** | Content Creation/YouTube | [mrbeast-skill](https://github.com/alchaincyf/mrbeast-skill) | `npx skills add alchaincyf/mrbeast-skill` |
| 🔥 **Trump** | Negotiation/Power/Communication | [trump-skill](https://github.com/alchaincyf/trump-skill) | `npx skills add alchaincyf/trump-skill` |
| ⭐ **Steve Jobs** | Product/Design/Strategy | [steve-jobs-skill](https://github.com/alchaincyf/steve-jobs-skill) | `npx skills add alchaincyf/steve-jobs-skill` |
| **Elon Musk** | Engineering/Cost/First Principles | [elon-musk-skill](https://github.com/alchaincyf/elon-musk-skill) | `npx skills add alchaincyf/elon-musk-skill` |
| **Munger** | Investment/Multidisciplinary Thinking | [munger-skill](https://github.com/alchaincyf/munger-skill) | `npx skills add alchaincyf/munger-skill` |
| **Feynman** | Learning/Teaching/Scientific Thinking | [feynman-skill](https://github.com/alchaincyf/feynman-skill) | `npx skills add alchaincyf/feynman-skill` |
| **Naval** | Wealth/Leverage/Life Philosophy | [naval-skill](https://github.com/alchaincyf/naval-skill) | `npx skills add alchaincyf/naval-skill` |
| **Taleb** | Risk/Antifragility/Uncertainty | [taleb-skill](https://github.com/alchaincyf/taleb-skill) | `npx skills add alchaincyf/taleb-skill` |
| **Zhang Xuefeng** | Education/Career/Class Mobility | [zhangxuefeng-skill](https://github.com/alchaincyf/zhangxuefeng-skill) | `npx skills add alchaincyf/zhangxuefeng-skill` |

### Topic Skill

| Topic | Domain | Standalone Repo | Install |
|------|------|---------|---------|
| **X Mentor** | X/Twitter Growth | [x-mentor-skill](https://github.com/alchaincyf/x-mentor-skill) | `npx skills add alchaincyf/x-mentor-skill` |

Person skills distill an individual's thinking; topic skills distill a domain's methodology. Each repo includes full research data and example conversations.

Want to distill someone not on the list? Install Persona Forge and say "Distill XXX".
Want to create a custom character? Say "Create a XXX persona".

---

## Darwin.skill: Evolve All Skills Continuously

<div align="center">

<a href="https://github.com/alchaincyf/darwin-skill">
<img src="https://raw.githubusercontent.com/alchaincyf/darwin-skill/master/assets/banner.svg" alt="Darwin.skill" width="600">
</a>

</div>

Persona Forge creates skills, **[Darwin](https://github.com/alchaincyf/darwin-skill)** evolves them.

Inspired by Karpathy's autoresearch, Darwin.skill uses autonomous experiment loops to batch-optimize all skills: 8-dimension evaluation, ratchet mechanism (only keep improvements, auto-rollback regressions), independent sub-agent scoring. Persona Forge's Phase 5 dual-agent refinement embeds Darwin's evaluation framework — one reason Persona Forge generates such high-quality skills.

```bash
npx skills add alchaincyf/darwin-skill
```

---

## How It Works

### Person Distillation Flow

Input a name, and Persona Forge does four things:

**1. Six parallel research streams** — writings, podcasts/interviews, social media, critic perspectives, decision records, life timeline. 6 agents run simultaneously, each archived.

**2. Triple-verification extraction** — a claim must pass three tests before being recorded as a mental model: appears across 2+ domains (not a one-off), can predict positions on new questions (has predictive power), not something any smart person would think (has exclusivity). All three required.

**3. Build the skill** — 3–7 mental models + 5–10 decision heuristics + expression DNA + values & anti-patterns + honest limits, written into SKILL.md.

**4. Quality validation** — test with 3 questions the person publicly answered; direction must match. Then test with 1 question they never addressed; skill should show appropriate uncertainty rather than false confidence.

### Persona Synthesis Flow

Input a character description, and:

**1. Dimension composition** — select/adjust dimensions from the Persona Dimension Library (teaching style, communication warmth, expression mode, interaction pattern, etc.).

**2. Domain knowledge injection** — 1 agent searches general domain methodology (e.g., "best practices for elementary math pedagogy").

**3. Character consistency design** — derive mental models, decision heuristics, and expression DNA from dimension composition + domain knowledge.

**4. Assembly & validation** — generate the skill using the extended template, then run character consistency tests, dimension deviation detection, and domain appropriateness tests.

Full methodology in `references/extraction-framework.md` (person distillation) and `references/persona-dimension-library.md` (persona synthesis).

---

## Repository Structure

```
persona-forge/
├── SKILL.md                      # Persona Forge core (3-path entry + index)
├── references/
│   ├── extraction-framework.md   # Extraction methodology (triple verification, quality checklist)
│   ├── skill-template.md         # Template for generating skills (person + synthetic persona)
│   ├── persona-dimension-library.md  # Persona Dimension Library (synthesis)
│   ├── path-a.md                 # Person distillation detailed flow (Phase 1-5)
│   └── path-c.md                 # Persona synthesis detailed flow (Phase 1C-5C)
└── examples/                          # 13 people + 1 topic
    ├── naval-perspective/             # Naval
    ├── elon-musk-perspective/         # Musk
    └── ...
```

All research is fully transparent. Each example includes complete research files — you can see how information was collected, filtered, and turned into mental models.

---

## About

Persona Forge doesn't copy people. It extracts cognitive operating systems.

A good person skill lets you see your own problems through another's eyes. A good persona character lets AI interact with you in the style and capability you need. Not to imitate them, but to expand your own thinking and interaction experience.

**Persona Forge** — a persona engine.

---

## License

MIT — use it, modify it, build with it.

---

<div align="center">

**Persona Forge** — Personify anything.

*The next mind you want to converse with doesn't have to be a famous one.*


</div>
