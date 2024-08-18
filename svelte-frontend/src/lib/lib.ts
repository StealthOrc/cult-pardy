



export function getHttpUrl(url: string): string {
    const isHttps = location.protocol === 'https:';
    return  `${isHttps ? 'https' : 'http'}://${location.host}${url}`;
}


