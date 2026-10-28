class_name WebImportBridge

const JAVASCRIPT := """
(function () {
	if (window.mindsphereImportInstalled) {
		return;
	}
	window.mindsphereImportInstalled = true;

	const style = document.createElement('style');
	style.textContent = `
#ms-import-root {
	position: fixed;
	inset: 0;
	z-index: 9999;
	display: none;
	align-items: center;
	justify-content: center;
	background: rgba(3, 7, 14, 0.74);
	backdrop-filter: blur(5px);
	font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
	color: #e8eef7;
}
#ms-import-root * { box-sizing: border-box; }
#ms-import-card {
	width: min(560px, calc(100vw - 48px));
	max-height: calc(100vh - 48px);
	overflow: auto;
	padding: 24px;
	border: 1px solid #1a2538;
	border-radius: 14px;
	background: #0b1322;
	box-shadow: 0 24px 80px rgba(0, 0, 0, 0.55);
}
#ms-import-card .ms-kicker {
	margin-bottom: 10px;
	color: #59d8e8;
	font-size: 11px;
	font-weight: 700;
	letter-spacing: 0.16em;
}
#ms-import-card h2 {
	margin: 0 0 10px;
	font-size: 22px;
	font-weight: 600;
}
#ms-import-card p {
	margin: 0 0 16px;
	color: #93a4bb;
	font-size: 13px;
	line-height: 1.55;
}
#ms-import-card code { color: #c7d5e8; }
#ms-import-file {
	width: 100%;
	margin-bottom: 10px;
	padding: 10px;
	border: 1px solid #1e2d44;
	border-radius: 8px;
	background: #08111f;
	color: #9fb0c7;
	font-size: 13px;
}
#ms-import-file::file-selector-button {
	margin-right: 10px;
	padding: 7px 11px;
	border: 1px solid #2a3c57;
	border-radius: 7px;
	background: #111d2f;
	color: #dce8f7;
	font: inherit;
	cursor: pointer;
}
#ms-import-json {
	display: block;
	width: 100%;
	height: 190px;
	padding: 10px;
	border: 1px solid #1e2d44;
	border-radius: 8px;
	background: #08111f;
	color: #dce8f7;
	font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
	font-size: 12px;
	line-height: 1.45;
	resize: vertical;
}
#ms-import-json:focus, #ms-import-file:focus {
	outline: none;
	border-color: #59d8e8;
}
#ms-import-error {
	min-height: 18px;
	margin-top: 8px;
	color: #ff8f7e;
	font-size: 12px;
}
#ms-import-card .ms-actions {
	display: flex;
	justify-content: flex-end;
	gap: 8px;
	margin-top: 14px;
}
#ms-import-card button {
	padding: 10px 16px;
	border-radius: 8px;
	font: inherit;
	font-size: 13px;
	font-weight: 600;
	cursor: pointer;
}
#ms-import-cancel {
	border: 1px solid transparent;
	background: transparent;
	color: #93a4bb;
}
#ms-import-cancel:hover { color: #e8eef7; }
#ms-import-submit {
	border: 1px solid #59d8e8;
	background: #59d8e8;
	color: #04121a;
}
#ms-import-submit:hover { background: #78e3f0; }
`;
	document.head.appendChild(style);

	const root = document.createElement('div');
	root.id = 'ms-import-root';
	root.innerHTML = `
<div id="ms-import-card" role="dialog" aria-modal="true" aria-labelledby="ms-import-title">
	<div class="ms-kicker">IMPORT</div>
	<h2 id="ms-import-title">Bring in a sphere</h2>
	<p>Choose a <code>.json</code> sphere, or paste its exported JSON below.</p>
	<input id="ms-import-file" type="file" accept=".json,application/json">
	<textarea id="ms-import-json" spellcheck="false" placeholder="Paste exported sphere JSON..."></textarea>
	<div id="ms-import-error"></div>
	<div class="ms-actions">
		<button id="ms-import-cancel" type="button">Cancel</button>
		<button id="ms-import-submit" type="button">Import sphere</button>
	</div>
</div>`;
	document.body.appendChild(root);

	const fileInput = root.querySelector('#ms-import-file');
	const textarea = root.querySelector('#ms-import-json');
	const error = root.querySelector('#ms-import-error');
	const cancel = root.querySelector('#ms-import-cancel');
	const submit = root.querySelector('#ms-import-submit');

	function close() {
		root.style.display = 'none';
		error.textContent = '';
	}

	function importText(text) {
		if (!String(text || '').trim()) {
			error.textContent = 'Choose a sphere file or paste its JSON first.';
			return;
		}
		if (typeof window.mindsphereReceiveImport !== 'function') {
			error.textContent = 'The import bridge is not ready. Reload the page and try again.';
			return;
		}
		close();
		window.mindsphereReceiveImport(String(text));
	}

	fileInput.addEventListener('change', function () {
		const file = fileInput.files && fileInput.files[0];
		if (!file) {
			return;
		}
		const reader = new FileReader();
		reader.onload = function () {
			textarea.value = String(reader.result || '');
			importText(textarea.value);
		};
		reader.onerror = function () {
			error.textContent = 'Could not read that file.';
		};
		reader.readAsText(file);
	});

	cancel.addEventListener('click', close);
	submit.addEventListener('click', function () {
		importText(textarea.value);
	});
	root.addEventListener('click', function (event) {
		if (event.target === root) {
			close();
		}
	});
	window.addEventListener('keydown', function (event) {
		if (event.key === 'Escape' && root.style.display === 'flex') {
			event.stopPropagation();
			close();
		}
	}, true);

	window.mindsphereOpenImport = function () {
		error.textContent = '';
		root.style.display = 'flex';
		window.setTimeout(function () { textarea.focus(); }, 0);
	};
})();
"""
