document.addEventListener('DOMContentLoaded', function() {
    // 添加复制按钮到代码块
    const codeBlocks = document.querySelectorAll('pre code');
    codeBlocks.forEach(function(block) {
        const pre = block.parentNode;
        const button = document.createElement('button');
        button.className = 'copy-button';
        button.textContent = '复制';
        button.onclick = function() {
            navigator.clipboard.writeText(block.textContent).then(function() {
                button.textContent = '已复制';
                setTimeout(function() {
                    button.textContent = '复制';
                }, 2000);
            });
        };
        
        // 设置按钮初始状态为隐藏
        button.style.opacity = '0';
        button.style.visibility = 'hidden';
        button.style.transition = 'opacity 0.2s ease, visibility 0.2s ease';
        
        pre.style.position = 'relative';
        pre.appendChild(button);
        
        // 鼠标进入代码块时显示按钮
        pre.addEventListener('mouseenter', function() {
            button.style.opacity = '0.7';
            button.style.visibility = 'visible';
        });
        
        // 鼠标离开代码块时隐藏按钮
        pre.addEventListener('mouseleave', function() {
            button.style.opacity = '0';
            button.style.visibility = 'hidden';
        });
        
        // 鼠标悬停在按钮上时提高透明度
        button.addEventListener('mouseenter', function() {
            button.style.opacity = '1';
        });
        
        button.addEventListener('mouseleave', function() {
            button.style.opacity = '0.7';
        });
    });
});
