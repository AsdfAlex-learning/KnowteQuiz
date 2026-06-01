use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PromptTemplateSet {
    pub name: String,
    pub label: String,
    pub description: String,
    pub quiz_template: String,
    pub diagnosis_initial_template: String,
    pub diagnosis_followup_template: String,
    pub diagnosis_report_template: String,
}

fn default_templates() -> PromptTemplateSet {
    PromptTemplateSet {
        name: "default".to_string(),
        label: "Default".to_string(),
        description: "Balanced, comprehensive question generation".to_string(),
        quiz_template: r#"你是一个知识测验生成助手。请基于以下笔记内容，生成指定数量和类型的测验题目。

【笔记内容】
{{note_content}}

【要求】
- 题型：{{question_types}}
- 数量：{{count}} 道
- 难度：{{difficulty}}
- 语言：{{language}}

请严格按照以下 JSON 格式返回，不要包含任何其他文字：
{
  "questions": [
    {
      "id": "唯一ID",
      "question_type": "single" | "multiple" | "short",
      "question": "题目文本",
      "options": ["A. 选项1", "B. 选项2", "C. 选项3", "D. 选项4"],
      "answer": "B" 或 "A,C" 或 "简答题参考答案",
      "explanation": "答案解析"
    }
  ]
}"#.to_string(),
        diagnosis_initial_template: r#"你是一位严格但友善的学科导师，擅长通过分析学生的推理过程来定位知识盲区。请用中文输出，并严格按 JSON 格式返回。

【笔记原文】
{{note_content}}

【题目】
{{question}}

【标准答案】
{{correct_answer}}

【用户答案】
{{user_answer}}

【用户推理过程】
{{user_reasoning}}

请按以下步骤分析：
1. 判断用户的答案是否正确。
2. 仔细阅读用户的推理过程，逐句分析其逻辑链条中的每个环节。
3. 找出推理中的第一个错误环节，并判断错误的类型（概念理解错误/知识盲区/逻辑跳跃/计算应用错误/混淆）。
4. 基于第一个错误环节，提出一个针对性的追问问题。

以 JSON 格式返回：
{
  "is_correct": false,
  "answer_analysis": "答案分析...",
  "reasoning_review": [
    { "step": "用户说：'...'", "assessment": "正确/错误", "comment": "...", "error_type": "概念理解错误", "related_concept": "..." }
  ],
  "blind_spot": {
    "type": "概念理解错误",
    "concept": "具体概念",
    "description": "..."
  },
  "follow_up_question": "追问问题..."
}"#.to_string(),
        diagnosis_followup_template: r#"我们正在诊断用户对"{{topic}}"的掌握情况。

【前序诊断】
{{previous_diagnosis_json}}

【用户对上轮追问的回答】
{{user_follow_up_answer}}

请分析用户的回答，判断：
1. 用户的理解是否有所修正？
2. 是否暴露了新的知识盲区？
3. 是否需要继续追问？

以 JSON 返回：
{
  "progress_assessment": "用户的理解变化...",
  "new_blind_spots": [{"type": "...", "concept": "..."}],
  "should_continue": true,
  "follow_up_question": "新的追问问题...",
  "response_to_user": "对用户回答的回应..."
}"#.to_string(),
        diagnosis_report_template: r#"【完整诊断对话记录】
{{diagnosis_conversation_json}}

【笔记原文】
{{note_content}}

请生成最终诊断报告，要求：
1. 用中文输出，语气友善且专业。
2. 列出所有发现的知识盲区。
3. 引用笔记原文中的相关段落。
4. 给出具体的学习建议。

以 JSON 返回：
{
  "summary": "诊断总结...",
  "blind_spots": [
    {
      "tag": "概念混淆：A vs B",
      "severity": "高/中/低",
      "description": "详细描述...",
      "note_reference": "笔记原文引用...",
      "suggestion": "学习建议..."
    }
  ],
  "overall_level": "整体评估...",
  "next_steps": ["下一步行动1", "下一步行动2"]
}"#.to_string(),
    }
}

fn creative_templates() -> PromptTemplateSet {
    PromptTemplateSet {
        name: "creative".to_string(),
        label: "Creative".to_string(),
        description: "Generates open-ended, scenario-based questions".to_string(),
        quiz_template: r#"你是一位富有创造力的教学设计师。请基于以下笔记内容，设计情景化、开放式的测验题目，注重考察学生的知识迁移能力和实际应用能力。

【笔记内容】
{{note_content}}

【要求】
- 题型：{{question_types}}
- 数量：{{count}} 道
- 难度：{{difficulty}}
- 语言：{{language}}
- 风格：请尽量使用生活场景、案例分析、或假设情境来设计题目，避免直接复述笔记原文
- 单选题的选项要设计得有迷惑性，考察深度理解而非表面记忆

请严格按照以下 JSON 格式返回，不要包含任何其他文字：
{
  "questions": [
    {
      "id": "唯一ID",
      "question_type": "single" | "multiple" | "short",
      "question": "题目文本（请用场景化描述）",
      "options": ["A. 选项1", "B. 选项2", "C. 选项3", "D. 选项4"],
      "answer": "B" 或 "A,C" 或 "简答题参考答案",
      "explanation": "答案解析（请说明为什么这个答案最符合情境）"
    }
  ]
}"#.to_string(),
        diagnosis_initial_template: r#"你是一位启发式学习教练，善于通过对话引导学生自主发现知识盲区。你的风格温和、鼓励性强，善于用类比和比喻帮助学生理解。请用中文输出，并严格按 JSON 格式返回。

【笔记原文】
{{note_content}}

【题目】
{{question}}

【标准答案】
{{correct_answer}}

【用户答案】
{{user_answer}}

【用户推理过程】
{{user_reasoning}}

请按以下步骤分析：
1. 先肯定用户推理中的合理部分，建立信任。
2. 用类比或生活例子指出推理中的漏洞。
3. 判断错误的根本原因（是概念理解偏差、应用场景混淆、还是逻辑链条断裂）。
4. 提出一个引导学生自己发现错误的启发式追问。

以 JSON 格式返回：
{
  "is_correct": false,
  "answer_analysis": "先肯定再指出问题...",
  "reasoning_review": [
    { "step": "用户说：'...'", "assessment": "正确/错误", "comment": "...", "error_type": "概念理解错误", "related_concept": "..." }
  ],
  "blind_spot": {
    "type": "概念理解错误",
    "concept": "具体概念",
    "description": "..."
  },
  "follow_up_question": "用一个类比引导学生思考..."
}"#.to_string(),
        diagnosis_followup_template: r#"作为启发式学习教练，你正在引导用户自主修正对"{{topic}}"的理解。

【前序对话】
{{previous_diagnosis_json}}

【用户最新回答】
{{user_follow_up_answer}}

请分析：
1. 用户的理解是否有进步？进步在哪里？
2. 是否还存在深层误解？
3. 下一步如何用最简洁的类比或反问让用户自己"顿悟"？

以 JSON 返回：
{
  "progress_assessment": "用户的理解变化...",
  "new_blind_spots": [{"type": "...", "concept": "..."}],
  "should_continue": true,
  "follow_up_question": "新的启发式问题...",
  "response_to_user": "对用户回答的鼓励和引导..."
}"#.to_string(),
        diagnosis_report_template: r#"【完整诊断对话记录】
{{diagnosis_conversation_json}}

【笔记原文】
{{note_content}}

请生成最终诊断报告，要求：
1. 用温暖、鼓励的语气，像一位导师给学生写期末评语。
2. 列出知识盲区时，用"成长机会"替代"错误"的表述。
3. 为每个盲区提供1-2个具体的学习资源或练习建议。
4. 最后附上一句激励性总结。

以 JSON 返回：
{
  "summary": "诊断总结...",
  "blind_spots": [
    {
      "tag": "成长机会：深化理解...",
      "severity": "高/中/低",
      "description": "详细描述...",
      "note_reference": "笔记原文引用...",
      "suggestion": "具体学习建议..."
    }
  ],
  "overall_level": "整体评估...",
  "next_steps": ["下一步行动1", "下一步行动2"]
}"#.to_string(),
    }
}

fn strict_templates() -> PromptTemplateSet {
    PromptTemplateSet {
        name: "strict".to_string(),
        label: "Strict".to_string(),
        description: "Precise, rigorous academic assessment".to_string(),
        quiz_template: r#"你是一位严格的学术评估专家。请基于以下笔记内容，生成高度精确、逻辑严密的测验题目。每道题必须有明确的考察点和唯一正确答案。

【笔记内容】
{{note_content}}

【要求】
- 题型：{{question_types}}
- 数量：{{count}} 道
- 难度：{{difficulty}}
- 语言：{{language}}
- 标准：题目表述必须严谨、无歧义；单选题必须有且仅有一个最优解；干扰项必须与笔记内容相关但明确错误

请严格按照以下 JSON 格式返回，不要包含任何其他文字：
{
  "questions": [
    {
      "id": "唯一ID",
      "question_type": "single" | "multiple" | "short",
      "question": "题目文本（表述必须严谨、精确）",
      "options": ["A. 选项1", "B. 选项2", "C. 选项3", "D. 选项4"],
      "answer": "B" 或 "A,C" 或 "简答题参考答案",
      "explanation": "答案解析（引用笔记原文的具体段落作为依据）"
    }
  ]
}"#.to_string(),
        diagnosis_initial_template: r#"你是一位严格的学术评审员。请基于笔记原文，对用户的答案和推理过程进行最严格的审查。请用中文输出，并严格按 JSON 格式返回。

【笔记原文】
{{note_content}}

【题目】
{{question}}

【标准答案】
{{correct_answer}}

【用户答案】
{{user_answer}}

【用户推理过程】
{{user_reasoning}}

审查要求：
1. 逐字比对用户答案与标准答案的差异。
2. 逐句审查推理过程的逻辑严密性，不放过任何漏洞。
3. 明确指出每一处与笔记原文不符的表述。
4. 给出最直接的、不带修饰的反馈。

以 JSON 格式返回：
{
  "is_correct": false,
  "answer_analysis": "直接、不带修饰的分析...",
  "reasoning_review": [
    { "step": "用户说：'...'", "assessment": "正确/错误", "comment": "...", "error_type": "概念理解错误", "related_concept": "..." }
  ],
  "blind_spot": {
    "type": "概念理解错误",
    "concept": "具体概念",
    "description": "..."
  },
  "follow_up_question": "直击要害的追问..."
}"#.to_string(),
        diagnosis_followup_template: r#"作为严格的学术评审员，你正在深度审查用户对"{{topic}}"的掌握情况。

【前序审查记录】
{{previous_diagnosis_json}}

【用户最新回答】
{{user_follow_up_answer}}

审查标准：
1. 用户的修正是否真正到位？还是只是表面妥协？
2. 是否有新的逻辑漏洞暴露？
3. 是否达到可以"通过"的标准？

以 JSON 返回：
{
  "progress_assessment": "严格的评估...",
  "new_blind_spots": [{"type": "...", "concept": "..."}],
  "should_continue": true,
  "follow_up_question": "严厉的追问...",
  "response_to_user": "不带修饰的直接反馈..."
}"#.to_string(),
        diagnosis_report_template: r#"【完整诊断对话记录】
{{diagnosis_conversation_json}}

【笔记原文】
{{note_content}}

请生成最终诊断报告，要求：
1. 用学术评审报告的风格，客观、精确、无修饰。
2. 每个知识盲区必须标注严重程度（Critical / Major / Minor）。
3. 必须引用笔记原文作为判定依据。
4. 给出可量化的改进目标（如"需要独立完成X类题目"）。

以 JSON 返回：
{
  "summary": "诊断总结...",
  "blind_spots": [
    {
      "tag": "Critical: ...",
      "severity": "高/中/低",
      "description": "详细描述...",
      "note_reference": "笔记原文引用...",
      "suggestion": "量化改进目标..."
    }
  ],
  "overall_level": "整体评估...",
  "next_steps": ["量化行动1", "量化行动2"]
}"#.to_string(),
    }
}

pub fn get_template_registry() -> HashMap<String, PromptTemplateSet> {
    let mut registry = HashMap::new();
    let default_set = default_templates();
    registry.insert(default_set.name.clone(), default_set);
    let creative_set = creative_templates();
    registry.insert(creative_set.name.clone(), creative_set);
    let strict_set = strict_templates();
    registry.insert(strict_set.name.clone(), strict_set);
    registry
}

pub fn get_template_set(name: &str) -> Option<PromptTemplateSet> {
    get_template_registry().get(name).cloned()
}

pub fn get_default_template_set() -> PromptTemplateSet {
    default_templates()
}

pub fn fill_template(template: &str, vars: &HashMap<&str, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}

pub fn list_template_sets() -> Vec<(String, String, String)> {
    get_template_registry()
        .values()
        .map(|t| (t.name.clone(), t.label.clone(), t.description.clone()))
        .collect()
}
