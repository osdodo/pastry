try {
    let ts = parseInt(input.trim());
    if (!isNaN(ts)) {
        if (ts < 10000000000) {
            ts *= 1000;
        }

        let d = new Date(ts);
        let pad = (n) => n.toString().padStart(2, '0');
        let s = `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
        output = s;
    } else {
        output = "Invalid Timestamp";
    }
} catch (e) {
    output = "Conversion Failed: " + e;
}
