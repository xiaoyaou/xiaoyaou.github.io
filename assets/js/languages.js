// 动态按需加载语言包
const prefix = 'https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/languages/';
const suffix = '.min.js';

// 检测页面中使用的语言
function detectLanguages() {
    const codeBlocks = document.querySelectorAll('div[class*="language-"], div[class*="lang-"]');
    const languages = new Set();

    codeBlocks.forEach(block => {
        const classList = Array.from(block.classList);
        classList.forEach(className => {
            if (className.startsWith('language-') || className.startsWith('lang-')) {
                const lang = className.replace(/^(language-|lang-)/, '');
                languages.add(lang);
            }
        });
    });
    return Array.from(languages);
}

// 动态加载语言包
function loadLanguage(language) {
    return new Promise((resolve, reject) => {
        if (typeof hljs !== 'undefined' && hljs.getLanguage(language)) {
            resolve();
            return;
        }

        const script = document.createElement('script');
        script.src = prefix + language + suffix;
        script.onload = resolve;
        script.onerror = reject;
        document.head.appendChild(script);
    });
}

// 页面加载完成后执行
document.addEventListener('DOMContentLoaded', function () {
    const languages = detectLanguages();

    if (languages.length === 0) {
        // 如果没有检测到特定语言，直接执行高亮
        hljs.highlightAll();
    } else {
        // 并行加载所需语言包
        Promise.all(languages.map(lang => loadLanguage(lang)))
            .then(() => {
                hljs.highlightAll();
            })
            .catch(error => {
                console.warn('语言包加载失败，使用默认高亮:', error);
                hljs.highlightAll();
            });
    }
});