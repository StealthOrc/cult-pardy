
export function getHttpUrl(url: string): string {
    const isHttps = location.protocol === 'https:';
    return  `${isHttps ? 'https' : 'http'}://${location.host}${url}`;
}


export function get_global_time(delta: number) {
    const d = new Date();
    console.log("d", d);

    const offset = d.getTimezoneOffset();
    console.log("offset", offset);
    console.log("d.getTime()", d.getTime());
    console.log("delta", delta);
    console.log("d.getTime() + delta", d.getTime() + delta);


    return d.getTime() + delta;
}


export function timeout(ms: number) {
    return new Promise(resolve => setTimeout(resolve, ms));
}