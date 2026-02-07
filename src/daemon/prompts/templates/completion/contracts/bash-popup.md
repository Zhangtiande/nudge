Shell mode: bash-popup
Return JSON only:
{"command":"<completed command>","summary_short":"<8-20 words concise explanation>","reason_short":"<short why>"}
Rules:
- command is required
- summary_short should be a short action-oriented description
- reason_short should explain why this candidate fits current input
- no markdown, no extra keys unless useful
- if JSON is not possible, return only the completed command text
