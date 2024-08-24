
export function getHttpUrl(url: string): string {
    const isHttps = location.protocol === 'https:';
    return  `${isHttps ? 'https' : 'http'}://${location.host}${url}`;
}


export function get_global_time(delta: number) {
    return new Date().getTime() + delta;
}


export function timeout(ms: number) {
    return new Promise(resolve => setTimeout(resolve, ms));
}