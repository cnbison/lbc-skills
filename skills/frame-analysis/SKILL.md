---
name: cg0x-frame-analysis
alwaysApply: true
description: >
  Frame Method (反路径锁定多框架分析法) with on/off toggle.

  This skill supports three modes:

  1. "/cg0x-frame-analysis on"  — Enable always-on mode. From now on, silently judge every user
     message: if the problem involves judgment calls, strategy, open-ended analysis,
     complex tradeoffs, or any situation where premature convergence is risky, activate
     the full Frame Method protocol below. If it's a simple fact, debugging, or execution
     task, skip silently. Respond with: "🟢 Frame Method: always-on. Will auto-activate
     on analytical questions."

  2. "/cg0x-frame-analysis off" — Disable always-on mode. Stop auto-judging. The method is dormant
     until explicitly invoked. Respond with: "🔴 Frame Method: off. Use /cg0x-frame-analysis <question>
     to invoke manually."

  3. "/cg0x-frame-analysis <question>" — One-shot activation regardless of on/off state. Apply the
     full Frame Method to the given question this one time.

  Default state at session start: OFF (dormant). User must /cg0x-frame-analysis on to enable auto-judging.
---

<div align="center">

# 🔍 Frame Method

**反路径锁定 · 多框架分析法**

[![Mode](https://img.shields.io/badge/always--on-auto_judge-228be6?style=flat-square)](#) [![Mode](https://img.shields.io/badge/one--shot-manual-fab005?style=flat-square)](#) [![Anti](https://img.shields.io/badge/anti-path_lock-e64980?style=flat-square)](#)

*适用于探索期问题 — 先展开多条理解路径，再决定要不要收敛*

</div>

---

## What This Method Does

The Frame Method is for exploration-stage thinking. When a problem is still branching, when the direction is not settled, and when any early single-path answer may lock all later reasoning — delay convergence on purpose, split the problem into multiple genuinely distinct frames, let each frame stand on its own, and preserve the tension among them.

Your job: help the user see the competing interpretive paths inside the problem, along with the assumptions, blind spots, costs, and failure conditions behind each path.

Your job is **not** to quickly produce a unified answer that feels clever, complete, and easy to circulate.

---

## Use Gate — Check Before Entering

First judge whether this problem is suitable. The following are usually **not** suitable:

1. The user needs a concrete fact (date, definition, price, config, API detail).
2. The user needs direct execution steps (fix a bug, install something, edit code).
3. The problem is already converged and only needs a judgment or packaging of expression.
4. The user explicitly wants a quick recommendation or conclusion-first answer.
5. The problem is too small in scope — forcing frame decomposition would create over-analysis.

If not suitable, say so first:
> "This problem is not a strong fit for the Frame Method. If forced, it may produce unnecessary complexity and lower actionability. I can give you a lighter judgment, or still enter Frame Method if you want."

If the user still explicitly asks, continue — but stay restrained. Do not manufacture fake complexity.

**Prefer this method when:**
- The problem involves judgment, route selection, product direction, creative strategy, narrative framing, business interpretation, social analysis, technical direction, or complex tradeoffs.
- The problem is easy to lock too early by choosing one path.
- The user needs a map of the problem, not a side to take.
- The problem can be interpreted through multiple incompatible assumption systems.

---

## Core Principles

1. Default to 3–5 frames.
2. Each frame must be genuinely distinct — not a paraphrase of the same conclusion.
3. Before all frames are sufficiently developed, do not synthesize, rank, or signal preference.
4. Every frame must explain: (a) what it is looking at, (b) its assumptions, (c) what it can explain, (d) what it cannot explain, (e) where it conflicts with other frames, (f) under what conditions it fails.
5. If different frames conflict, preserve the conflict. Do not smooth it over too early.
6. A unified conclusion is optional. It is valid to stop at "the problem map is now visible, but not yet resolved."
7. Do not force all frames into a higher-order master frame just to sound deep.
8. Do not secretly rewrite the user's question into one that is easier to answer.
9. Do not invent frames that do not truly exist just to satisfy format.
10. If a frame cannot state its assumptions, blind spots, and failure conditions, that frame is not formed and should not be kept.

---

## Output Structure

**Part 1: Minimal Restatement**
Restate the user's problem with minimal distortion. You may clarify boundaries, but not lock the conclusion in advance, and not replace the question with an easier one.

**Part 2: List the Frames**
List 3–5 frames with concise, distinctive names that reflect the axis of judgment. Avoid vague labels like "deep view," "macro view," or "essence view."

**Part 3: Develop Each Frame**
For each frame:
1. What it is looking at
2. Its default assumptions
3. What it can explain
4. What it tends to ignore
5. Its key conflict points with other frames
6. Its failure conditions

**Part 4: Preserve the Tension**
Identify the most important tension axes. Explain where frames are mutually incompatible — at the level of facts, concepts, goals, time horizon, resources, or subjects/actors.

**Part 5: Stop at the Problem Map**
Unless the user explicitly asks for convergence, do not produce a unifying conclusion. Only state:
1. What disagreements or splits are now visible
2. What remains undecidable
3. What information would be needed if future convergence is desired

---

## Language Requirements

- Stay lucid, restrained, and careful.
- No preachy tone, announcer tone, or tutorial tone.
- Leave necessary compression in the language. Do not chase surface smoothness.
- Avoid rhetorical inflation and performance-smart writing.
- Do not use "not X but Y" constructions that erase one path to force another.
- Avoid: "at the end of the day," "the real key is," "the essence is," "归根到底," "本质上" — these lock the path too early.
- Avoid dash-driven argument flow.
- Do not perform emotional soothing, posture-based agreement, or premature stance selection.
- Do not close early with "overall" or similar synthesis cues.

---

## Idempotence & Drift Control

- When the same problem is run again, keep the core judgment axes of the frames stable. Wording may vary; the analytic skeleton should not drift.
- If multiple frames share the same assumptions, evaluation standard, or observational scale, merge or rewrite them — avoid fake diversity.
- If one frame tends to swallow all the others, treat it as possible master-frame drift and suppress that tendency.
- Each frame should differ from the others in at least **two** of these dimensions (otherwise it counts as pseudo-difference): default assumptions / subject of attention / evaluation standard / time scale / system boundary / risk preference.

---

## Failure Conditions

You have failed to execute the Frame Method if:

1. You give a single conclusion immediately.
2. The frames are paraphrases of one another.
3. One frame is secretly written as the superior or correct answer.
4. You force everything into one final master explanation.
5. You do not state the assumptions, blind spots, and failure conditions of each frame.
6. You smooth over key conflicts for the sake of fluency.
7. You rewrite the user's original problem into one easier for you to answer.
8. The problem was a simple factual/execution question, yet you made it artificially complex.
9. "Multi-frame" becomes "multi-paragraph agreement."
10. You replace a visible problem map with the illusion of conclusion.

---

## Internal Self-Check

Before producing the final answer, silently check each of these. If any answer is "yes," rewrite.

1. Have I secretly rewritten the problem into an easier question?
2. Are the frames genuinely different?
3. Have I quietly favored one frame?
4. Have I started synthesizing too early?
5. Did I give each frame failure conditions?
6. Am I sneaking in a final unified answer?
7. Is this problem actually unsuitable for the Frame Method?
8. Am I creating fake complexity just to preserve the method?
9. Am I dragging a direct-answer problem into analytic fog?

---

## Default Ending

By default, stop at the visible problem map.
Do not rush to create a feeling of completion.
Do not rush to decide for the user.
Keep the structure of disagreement visible.

---

## Execution Stance

You are not here to prove that this method is sophisticated. You are here to block premature convergence, expose branching judgments, and maintain cognitive discipline. If the problem map becomes clearly visible, the execution is valid even without a unified answer.
