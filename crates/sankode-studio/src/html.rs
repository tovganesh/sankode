pub const STUDIO_HTML: &str = r#"<!DOCTYPE html>
<html lang="sa">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>॥ सङ्कोड वेधशाला ॥ Sankode Studio IDE</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Fira+Code:wght@400;500;600&family=Noto+Sans+Devanagari:wght@400;500;600;700&family=Outfit:wght@400;500;600;700&display=swap" rel="stylesheet">
    <style>
        :root {
            --bg-base: #090d16;
            --bg-surface: #0f172a;
            --bg-elevated: #1e293b;
            --border: #334155;
            --border-focus: #f59e0b;
            --text-main: #f8fafc;
            --text-muted: #94a3b8;
            --saffron: #f59e0b;
            --saffron-dark: #d97706;
            --green: #10b981;
            --red: #ef4444;
            --cyan: #06b6d4;
            --font-dev: 'Noto Sans Devanagari', 'Fira Code', monospace;
            --font-ui: 'Outfit', sans-serif;
        }

        * {
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }

        body {
            background-color: var(--bg-base);
            color: var(--text-main);
            font-family: var(--font-ui);
            height: 100vh;
            display: flex;
            flex-direction: column;
            overflow: hidden;
        }

        /* Header / Navbar */
        header {
            background-color: rgba(15, 23, 42, 0.85);
            backdrop-filter: blur(12px);
            border-bottom: 1px solid var(--border);
            padding: 0.75rem 1.5rem;
            display: flex;
            align-items: center;
            justify-content: space-between;
            z-index: 10;
        }

        .brand {
            display: flex;
            align-items: center;
            gap: 0.75rem;
        }

        .brand-logo {
            font-size: 1.5rem;
            background: linear-gradient(135deg, #f59e0b, #ef4444);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            font-weight: 700;
            font-family: var(--font-dev);
            letter-spacing: 0.5px;
        }

        .brand-sub {
            font-size: 0.75rem;
            color: var(--text-muted);
            text-transform: uppercase;
            letter-spacing: 1.5px;
        }

        .toolbar {
            display: flex;
            align-items: center;
            gap: 0.75rem;
        }

        .btn {
            background: var(--bg-elevated);
            color: var(--text-main);
            border: 1px solid var(--border);
            padding: 0.5rem 1rem;
            border-radius: 6px;
            font-size: 0.875rem;
            font-weight: 500;
            cursor: pointer;
            display: flex;
            align-items: center;
            gap: 0.4rem;
            transition: all 0.2s ease;
        }

        .btn:hover {
            border-color: var(--saffron);
            background: rgba(245, 158, 11, 0.1);
        }

        .btn-primary {
            background: linear-gradient(135deg, var(--saffron), var(--saffron-dark));
            color: #000;
            font-weight: 600;
            border: none;
        }

        .btn-primary:hover {
            background: linear-gradient(135deg, #fbbf24, var(--saffron));
            box-shadow: 0 0 15px rgba(245, 158, 11, 0.4);
        }

        .btn-script {
            background: linear-gradient(135deg, #06b6d4, #0284c7);
            color: #fff;
            font-weight: 600;
            border: none;
        }

        .btn-script:hover {
            box-shadow: 0 0 15px rgba(6, 182, 212, 0.4);
        }

        .btn-check {
            border-color: var(--green);
            color: var(--green);
        }

        .btn-check:hover {
            background: rgba(16, 185, 129, 0.15);
        }

        select.example-select {
            background: var(--bg-elevated);
            color: var(--text-main);
            border: 1px solid var(--border);
            padding: 0.5rem 0.75rem;
            border-radius: 6px;
            font-size: 0.875rem;
            outline: none;
            cursor: pointer;
            font-family: var(--font-dev);
        }

        /* IME Toggle Switch */
        .ime-toggle-wrap {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            background: var(--bg-elevated);
            padding: 0.35rem 0.75rem;
            border-radius: 20px;
            border: 1px solid var(--border);
            font-size: 0.8rem;
        }

        .switch {
            position: relative;
            display: inline-block;
            width: 34px;
            height: 18px;
        }

        .switch input {
            opacity: 0;
            width: 0;
            height: 0;
        }

        .slider {
            position: absolute;
            cursor: pointer;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background-color: #475569;
            transition: .3s;
            border-radius: 18px;
        }

        .slider:before {
            position: absolute;
            content: "";
            height: 12px;
            width: 12px;
            left: 3px;
            bottom: 3px;
            background-color: white;
            transition: .3s;
            border-radius: 50%;
        }

        input:checked + .slider {
            background-color: var(--saffron);
        }

        input:checked + .slider:before {
            transform: translateX(16px);
        }

        /* Main Workspace Container */
        main {
            flex: 1;
            display: grid;
            grid-template-columns: 1.1fr 0.9fr;
            height: calc(100vh - 61px);
            overflow: hidden;
        }

        /* Editor Section */
        .editor-pane {
            display: flex;
            flex-direction: column;
            border-right: 1px solid var(--border);
            background: var(--bg-surface);
            position: relative;
        }

        .pane-header {
            background: rgba(15, 23, 42, 0.6);
            border-bottom: 1px solid var(--border);
            padding: 0.5rem 1rem;
            display: flex;
            align-items: center;
            justify-content: space-between;
            font-size: 0.8rem;
            color: var(--text-muted);
        }

        .editor-container {
            flex: 1;
            display: flex;
            position: relative;
            overflow: hidden;
            font-family: var(--font-dev);
            font-size: 1.05rem;
            line-height: 1.6;
        }

        .line-numbers {
            width: 45px;
            padding: 1rem 0.5rem;
            background: rgba(11, 15, 25, 0.6);
            color: #475569;
            text-align: right;
            user-select: none;
            font-family: 'Fira Code', monospace;
            font-size: 0.85rem;
            border-right: 1px solid rgba(51, 65, 85, 0.4);
            line-height: 1.6;
        }

        textarea#code-editor {
            flex: 1;
            background: transparent;
            color: var(--text-main);
            border: none;
            outline: none;
            padding: 1rem;
            resize: none;
            font-family: var(--font-dev);
            font-size: 1.05rem;
            line-height: 1.6;
            tab-size: 4;
            white-space: pre;
            overflow-y: auto;
        }

        /* Output & Console Section */
        .console-pane {
            display: flex;
            flex-direction: column;
            background: var(--bg-base);
        }

        .tab-bar {
            display: flex;
            background: var(--bg-surface);
            border-bottom: 1px solid var(--border);
        }

        .tab-btn {
            padding: 0.6rem 1.2rem;
            font-size: 0.85rem;
            background: transparent;
            color: var(--text-muted);
            border: none;
            border-bottom: 2px solid transparent;
            cursor: pointer;
            font-family: var(--font-ui);
            font-weight: 500;
        }

        .tab-btn.active {
            color: var(--saffron);
            border-bottom-color: var(--saffron);
            background: rgba(245, 158, 11, 0.05);
        }

        .console-content {
            flex: 1;
            padding: 1rem;
            font-family: var(--font-dev);
            font-size: 0.95rem;
            line-height: 1.5;
            overflow-y: auto;
            white-space: pre-wrap;
            color: #cbd5e1;
        }

        .console-content.success {
            color: #34d399;
        }

        .console-content.error {
            color: #f87171;
        }

        .status-bar {
            background: var(--bg-surface);
            border-top: 1px solid var(--border);
            padding: 0.35rem 1rem;
            font-size: 0.75rem;
            color: var(--text-muted);
            display: flex;
            justify-content: space-between;
        }

        /* Keyboard Shortcut Hint */
        .kbd {
            background: rgba(255, 255, 255, 0.1);
            padding: 0.1rem 0.4rem;
            border-radius: 4px;
            font-size: 0.75rem;
            font-family: 'Fira Code', monospace;
        }
    </style>
</head>
<body>
    <header>
        <div class="brand">
            <span class="brand-logo">॥ सङ्कोड वेधशाला ॥</span>
            <span class="brand-sub">Sankode Studio v०.१.०</span>
        </div>

        <div class="toolbar">
            <select id="example-picker" class="example-select" onchange="loadExample(this.value)">
                <option value="hello">१. नमस्ते जगत् (Hello World)</option>
                <option value="fibonacci">२. फिबोनाची गणना (Fibonacci)</option>
                <option value="ownership">३. स्वामित्वम् एवं ऋणम् (Ownership & Borrowing)</option>
                <option value="script">४. सङ्स्कृ लिपिः (Python-style Script)</option>
            </select>

            <div class="ime-toggle-wrap">
                <span title="Type Roman keys (e.g. kriya) and auto-convert to Devanagari">लिपि-परिवर्तक (IME)</span>
                <label class="switch">
                    <input type="checkbox" id="ime-toggle" onchange="toggleIME(this.checked)">
                    <span class="slider"></span>
                </label>
            </div>

            <button class="btn btn-check" onclick="checkCode()" title="Verify static types and borrow safety">
                <span>🛡️</span> सत्यापय
            </button>

            <button class="btn btn-script" onclick="runSanskipt()" title="Run as dynamic Python-style script">
                <span>⚡</span> सङ्स्कृ
            </button>

            <button class="btn btn-primary" onclick="runSankode()" title="Compile, check safety and execute (Ctrl+Enter)">
                <span>▶</span> सङ्कोड
            </button>
        </div>
    </header>

    <main>
        <!-- Code Editor -->
        <section class="editor-pane">
            <div class="pane-header">
                <span id="editor-filename">कार्यक्रमम्.सङ्</span>
                <span><span class="kbd">Ctrl+Enter</span> सञ्चालनार्थम्</span>
            </div>
            <div class="editor-container">
                <div class="line-numbers" id="line-numbers">1</div>
                <textarea id="code-editor" spellcheck="false" placeholder="अत्र सङ्केतं लिखन्तु..."></textarea>
            </div>
            <div class="status-bar">
                <span id="cursor-pos">पङ्क्तिः: १ | स्तम्भः: १</span>
                <span id="status-tag">सुरक्षा: अप्ररीक्षिता</span>
            </div>
        </section>

        <!-- Console & Output Pane -->
        <section class="console-pane">
            <div class="tab-bar">
                <button class="tab-btn active" onclick="switchTab('output')">फलम् (Output)</button>
                <button class="tab-btn" onclick="switchTab('ast')">वाक्यवृक्षः (AST)</button>
                <button class="tab-btn" onclick="switchTab('tokens')">सङ्केताः (Tokens)</button>
            </div>
            <div class="console-content" id="console-output">॥ सङ्कोड वेधशालायै स्वागताः ॥
सङ्केतस्य सञ्चालनार्थम् उपरिस्थे '▶ सङ्कोड' अथवा '⚡ सङ्स्कृ' कुञ्जिकायां नुदन्तु।</div>
            <div class="status-bar">
                <span id="exec-time">समयः: ०ms</span>
                <button class="btn" style="padding: 0.1rem 0.5rem; font-size: 0.7rem;" onclick="clearConsole()">मार्जय (Clear)</button>
            </div>
        </section>
    </main>

    <script>
        const EXAMPLES = {
            hello: `॥ नमस्ते जगत् - सङ्कोडस्य प्रथमं कार्यक्रमम् ॥

क्रिया मुख्य() -> रिक्त
    मुद्रय("नमस्ते जगत्!")।
    मान गणना = १०।
    मुद्रय("गणना = ", गणना)।
इति`,
            fibonacci: `॥ फिबोनाची गणना - सङ्कोडप्रदर्शनम् ॥

क्रिया फिबोनाची(संख्या: पूर्ण६४) -> पूर्ण६४
    यदि संख्या <= १
        प्रति संख्या।
    इति
    प्रति फिबोनाची(संख्या - १) + फिबोनाची(संख्या - २)।
इति

क्रिया मुख्य() -> रिक्त
    मान परिणाम = फिबोनाची(१०)।
    मुद्रय("फिबोनाची(१०) = ", परिणाम)।
इति`,
            ownership: `॥ स्वामित्वम् एवं ऋणग्रहणम् - सङ्कोडस्य स्मृति-सुरक्षा ॥

क्रिया मुख्य() -> रिक्त
    मान विकार्य मूलधन = १०००।
    मान साक्षी = ऋण मूलधन।
    मुद्रय("साक्षिणा दृष्टं मूलधनम् = ", साक्षी)।
इति`,
            script: `॥ सङ्स्कृ लिपिः - अजगरस्य (Python) इव सरलं सङ्केतनम् ॥

मान देश = "भारतम्"।
मुद्रय("नमस्ते ", देश)।

मान संख्या = १०।
मान वर्ग = संख्या * संख्या।
मुद्रय("संख्यायाः वर्गः = ", वर्ग)।

मान क = २५।
मान ख = ३५।
मुद्रय("योगः = ", क + ख)।`
        };

        const editor = document.getElementById('code-editor');
        const lineNumbers = document.getElementById('line-numbers');
        const consoleOutput = document.getElementById('console-output');
        const statusTag = document.getElementById('status-tag');
        const execTime = document.getElementById('exec-time');
        let currentTab = 'output';
        let currentAst = '';
        let currentTokens = '';
        let imeEnabled = false;

        function updateLineNumbers() {
            const lines = editor.value.split('\n').length;
            lineNumbers.innerHTML = Array.from({length: lines}, (_, i) => i + 1).join('<br>');
        }

        editor.addEventListener('input', () => {
            updateLineNumbers();
        });

        // Tab key support in textarea
        editor.addEventListener('keydown', (e) => {
            if (e.key === 'Tab') {
                e.preventDefault();
                const start = editor.selectionStart;
                const end = editor.selectionEnd;
                editor.value = editor.value.substring(0, start) + "    " + editor.value.substring(end);
                editor.selectionStart = editor.selectionEnd = start + 4;
            } else if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
                e.preventDefault();
                runSankode();
            }
        });

        // Live Phonetic Transliteration inside Editor if IME enabled
        editor.addEventListener('keyup', async (e) => {
            if (!imeEnabled) return;
            // Transliterate on space, enter, or danda
            if (e.key === ' ' || e.key === 'Enter' || e.key === '|') {
                const start = editor.selectionStart;
                const text = editor.value;
                // Find start of previous word
                let wordStart = start - 2;
                while (wordStart >= 0 && !/\s/.test(text[wordStart])) {
                    wordStart--;
                }
                wordStart++;
                const word = text.substring(wordStart, start - 1);
                if (word.length > 0 && /^[a-zA-Z0-9|]+$/.test(word)) {
                    try {
                        const res = await fetch('/api/transliterate', {
                            method: 'POST',
                            headers: { 'Content-Type': 'application/json' },
                            body: JSON.stringify({ text: word })
                        });
                        const data = await res.json();
                        if (data.devanagari) {
                            const newText = text.substring(0, wordStart) + data.devanagari + text.substring(start - 1);
                            editor.value = newText;
                            const diff = data.devanagari.length - word.length;
                            editor.selectionStart = editor.selectionEnd = start + diff;
                            updateLineNumbers();
                        }
                    } catch (err) {}
                }
            }
        });

        function toggleIME(val) {
            imeEnabled = val;
        }

        function loadExample(key) {
            if (EXAMPLES[key]) {
                editor.value = EXAMPLES[key];
                document.getElementById('editor-filename').innerText = key === 'script' ? 'लिपिः.सङ्स्कृ' : 'कार्यक्रमम्.सङ्';
                updateLineNumbers();
                statusTag.innerText = "सुरक्षा: अप्ररीक्षिता";
                statusTag.style.color = "var(--text-muted)";
            }
        }

        async function runSankode() {
            await executeCode('sankode');
        }

        async function runSanskipt() {
            await executeCode('sanskipt');
        }

        async function executeCode(mode) {
            const code = editor.value;
            consoleOutput.className = 'console-content';
            consoleOutput.innerText = "निष्पाद्यते... (Executing...)";
            const start = performance.now();

            try {
                const res = await fetch('/api/run', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ code, mode })
                });
                const data = await res.json();
                const duration = Math.round(performance.now() - start);
                execTime.innerText = `समयः: ${duration}ms`;

                currentAst = data.ast || '';
                currentTokens = data.tokens || '';

                if (data.success) {
                    consoleOutput.className = 'console-content success';
                    consoleOutput.innerText = data.output || "सफलतया निष्पादितम् (Executed successfully, no stdout)";
                    statusTag.innerText = "सुरक्षा: निर्दोषः ✓";
                    statusTag.style.color = "var(--green)";
                } else {
                    consoleOutput.className = 'console-content error';
                    consoleOutput.innerText = data.error || data.output || "दोषः जातः (Error occurred)";
                    statusTag.innerText = "सुरक्षा: सदोषः ✗";
                    statusTag.style.color = "var(--red)";
                }

                if (currentTab !== 'output') {
                    switchTab(currentTab);
                }
            } catch (e) {
                consoleOutput.className = 'console-content error';
                consoleOutput.innerText = "सञ्चारदोषः (Network/server error): " + e.message;
            }
        }

        async function checkCode() {
            const code = editor.value;
            consoleOutput.className = 'console-content';
            consoleOutput.innerText = "परीक्ष्यते... (Checking types & borrow safety...)";

            try {
                const res = await fetch('/api/check', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ code })
                });
                const data = await res.json();

                if (data.valid) {
                    consoleOutput.className = 'console-content success';
                    consoleOutput.innerText = "✓ निर्दोषः सङ्केतः!\nप्रकारदोषः स्वामित्वदोषो वा न प्राप्तः। (Static types & borrow safety verified successfully!)";
                    statusTag.innerText = "सुरक्षा: निर्दोषः ✓";
                    statusTag.style.color = "var(--green)";
                } else {
                    consoleOutput.className = 'console-content error';
                    consoleOutput.innerText = data.error;
                    statusTag.innerText = "सुरक्षा: सदोषः ✗";
                    statusTag.style.color = "var(--red)";
                }
            } catch (e) {
                consoleOutput.className = 'console-content error';
                consoleOutput.innerText = "परीक्षणे दोषः: " + e.message;
            }
        }

        function switchTab(tab) {
            currentTab = tab;
            document.querySelectorAll('.tab-btn').forEach(btn => btn.classList.remove('active'));
            if (tab === 'output') {
                document.querySelectorAll('.tab-btn')[0].classList.add('active');
            } else if (tab === 'ast') {
                document.querySelectorAll('.tab-btn')[1].classList.add('active');
                consoleOutput.className = 'console-content';
                consoleOutput.innerText = currentAst || "वाक्यवृक्षं द्रष्टुं पूर्वं सञ्चालनम् अनुतिष्ठन्तु।";
            } else if (tab === 'tokens') {
                document.querySelectorAll('.tab-btn')[2].classList.add('active');
                consoleOutput.className = 'console-content';
                consoleOutput.innerText = currentTokens || "सङ्केतान् द्रष्टुं पूर्वं सञ्चालनम् अनुतिष्ठन्तु।";
            }
        }

        function clearConsole() {
            consoleOutput.className = 'console-content';
            consoleOutput.innerText = "";
        }

        // Initialize with Hello World
        loadExample('hello');
    </script>
</body>
</html>
"#;
