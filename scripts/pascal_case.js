const words = input.trim().split(/[\s_\-]+/);
output = words.map(w => w.charAt(0).toUpperCase() + w.slice(1).toLowerCase()).join('');

