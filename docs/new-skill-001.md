很好，这一步其实是在做一件更“工程化”的事情：

> 👉 把“概念解剖能力”产品化成一个 **可复用 Skill（认知工具）**

我给你一版可以直接用于 **Agent / Prompt / Skill模块** 的设计，不是泛泛方法，而是**结构化、可执行、可扩展的版本**。

---

# 🧠 Skill 名称

**Chinese Concept Deep Decomposition（中文概念深度解剖）**

---

# 🎯 Skill 目标

对任意中文概念进行系统性拆解，输出：

* 语义结构
* 文化编码
* 行为指向
* 运行机制

👉 最终形成一个：

> **“可解释 + 可操作 + 可建模”的概念模型**

---

# 🧩 Skill 输入

```yaml
concept: <中文概念>
context: <可选：使用场景，如商业/社会/AI等>
depth: <分析深度：basic / deep / research>
```

---

# ⚙️ Skill 执行流程（核心）

## Step 1：构词与词源拆解（Lexical Layer）

### 要求：

* 拆分每个字
* 分析组合关系（并列 / 修饰 / 隐喻 / 动宾）

### 输出：

```yaml
structure:
  characters:
    - 字: 含义
  composition: <组合关系>
  literal_meaning: <字面含义>
```

---

## Step 2：语义核心提炼（Semantic Core）

### 要求：

* 不给“定义”，而是总结“实际用法”
* 提炼 2–4 个核心语义维度

### 输出：

```yaml
semantic_core:
  - 维度1: 描述
  - 维度2: 描述
```

---

## Step 3：语境依赖分析（Context Layer）

### 要求：

* 至少分析 3 个不同语境（如日常 / 商业 / 权力）
* 指出含义如何变化

### 输出：

```yaml
contexts:
  - 场景: 含义变化
  - 场景: 含义变化
```

---

## Step 4：文化与隐含规则（Cultural Encoding）

### 要求：

* 识别背后的文化机制（如人情、面子、权力、集体主义）
* 判断是否属于“非正式制度”

### 输出：

```yaml
cultural_layer:
  implicit_rules:
    - 规则1
    - 规则2
  system_type: <formal / informal / hybrid>
```

---

## Step 5：行为与操作映射（Action Mapping）

### 要求：

* 把概念转化为“动词”
* 描述现实中如何被“使用”

### 输出：

```yaml
actions:
  - 行为1: 描述
  - 行为2: 描述
```

---

## Step 6：结构与关系网络（Relational Structure）

### 要求：

* 是否构成网络？
* 是否存在层级 / 非对称？

### 输出：

```yaml
structure_model:
  type: <network / hierarchy / hybrid>
  properties:
    - 对称性: yes/no
    - 是否可量化: yes/no
```

---

## Step 7：边界与模糊性（Boundary Analysis）

### 要求：

* 概念的边界在哪里？
* 容易混淆的近义概念

### 输出：

```yaml
boundaries:
  includes: []
  excludes: []
  confusions:
    - 概念A: 区别
```

---

## Step 8：一句话本质抽象（Core Abstraction）

### 要求：

* 提炼一个“非语言依赖”的本质描述

### 输出：

```yaml
essence: <一句话抽象定义>
```

---

# 🧱 Skill 输出（完整结构）

最终输出应该是一个结构化对象：

```yaml
concept_model:
  name: ""
  structure: {}
  semantic_core: []
  contexts: []
  cultural_layer: {}
  actions: []
  structure_model: {}
  boundaries: {}
  essence: ""
```

---

# 🧠 Prompt 模板（可直接用）

你可以直接用于 Claude / GPT / Agent：

---

你是一个“中文概念解剖专家”，请对给定概念进行系统性拆解。

【目标】
不是解释概念，而是建立一个“可操作的概念模型”。

【分析步骤】

1. 构词与词源拆解（字面结构）
2. 提炼语义核心（不是定义，而是使用方式）
3. 分析不同语境下的含义变化
4. 挖掘文化隐含与潜规则
5. 映射到现实行为（这个概念如何被使用）
6. 分析其结构类型（网络 / 层级 / 非对称）
7. 明确边界与易混淆概念
8. 提炼一句话本质（去语言化）

【输出要求】
使用结构化格式输出，避免泛泛解释。

【输入概念】
{{concept}}

---

