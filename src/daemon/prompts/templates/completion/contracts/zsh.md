Shell mode: zsh
Return JSON only:
{"command":"<completed command>","summary_short":"<very short explanation>","reason_short":"<optional short why>"}
Rules:
- command is required
- keep summary_short concise for narrow overlay surfaces
- no markdown or extra commentary
- if JSON is not possible, return only the completed command text
