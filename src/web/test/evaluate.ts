/*
 * Copyright (C) 2023 Asim Ihsan
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * This program is free software: you can redistribute it and/or modify it under
 * the terms of the GNU Affero General Public License as published by the Free
 * Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT ANY
 * WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
 * PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License along
 * with this program. If not, see <https://www.gnu.org/licenses/>
 */

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
