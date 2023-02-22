const puppeteer = require('puppeteer');

const listenPageErrors = async (page) => {
    // listen to browser console there
    page.on('console', async (message) => {
        const type = message.type().substr(0, 3).toUpperCase();
        console.log(`console.${type}: ${message.text()}`);
    });
}


(async () => {
    const browser = await puppeteer.launch();
    const page = await browser.newPage();
    await listenPageErrors(page);
    await page.goto('http://localhost:8086');
    await page.screenshot({ path: 'example.png' });
    await browser.close();
})();
