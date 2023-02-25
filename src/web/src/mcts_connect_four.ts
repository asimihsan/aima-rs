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

// @ts-ignore
import Phaser from 'phaser';

import { GameWrapper } from './pkg_mcts_connect_four';

const importPromise = import('./pkg_mcts_connect_four/mcts_connect_four');

export default function importPromiseMctsConnectFour() {
    return importPromise;
}

class MyScene extends Phaser.Scene {
    // width is number of columns in the board.
    width: number;

    // height is number of rows in the board.
    height: number;

    // squareSizePx is the size of each square in the board.
    squareSizePx: number;

    // squarePaddingPx is the padding between each square in the board.
    squarePaddingPx: number;

    // backgroundRects is an array of Phaser.GameObjects.Rectangle that are used to draw the board. Tokens for players
    // will be drawn on top of these.
    backgroundRects: Phaser.GameObjects.Rectangle[];

    add: Phaser.GameObjects.GameObjectFactory;
    load: Phaser.Loader.LoaderPlugin;
    tweens: Phaser.Tweens.TweenManager;

    // constructor that takes a config object and populates fields.
    constructor(
        {
            width = 800,
            height = 600,
            squareSizePx = 100,
            squarePaddingPx = 10,
        } = {},
    ) {
        super({ key: 'MyScene' });
        this.width = width;
        this.height = height;
        this.squareSizePx = squareSizePx;
        this.squarePaddingPx = squarePaddingPx;
        this.backgroundRects = [];

        for (let i = 0; i < this.width * this.height; i += 1) {
            this.backgroundRects.push(null);
        }
    }

    create() {
        // fill backgroundRects with Phaser.GameObjects.Rectangle objects of squareSizePx padded by squarePaddingPx in
        // the correct locations. calling this.add.rectangle(x, y, width, height, 0xffffff) will create a rectangle.
        const initialX = this.squareSizePx / 2 + this.squarePaddingPx;
        const initialY = this.squareSizePx / 2 + this.squarePaddingPx;
        for (let i = 0; i < this.width; i += 1) {
            for (let j = 0; j < this.height; j += 1) {
                const x = initialX + i * (this.squareSizePx + this.squarePaddingPx);
                const y = initialY + j * (this.squareSizePx + this.squarePaddingPx);
                this.backgroundRects[i * this.width + j] = this.add.rectangle(
                    x,
                    y,
                    this.squareSizePx,
                    this.squareSizePx,
                    0xffffff,
                );
            }
        }
        // const logo = this.add.image(400, 70, 'logo');
        // this.tweens.add({
        //     targets: logo,
        //     y: 500,
        //     duration: 2000,
        //     ease: 'Power2',
        //     yoyo: true,
        //     loop: -1,
        //
        // });
    }
}

class GameWorker {
    worker: Worker;
    isReady: boolean;
    messageQueue: any[];

    constructor() {
        this.isReady = false;
        this.messageQueue = [];
        this.worker = new Worker(new URL('./mcts_connect_four_worker.ts', import.meta.url));
        this.worker.onmessage = (event) => {
            if (event.data.type === 'ready') {
                console.log('Worker is ready');
                this.isReady = true;
                return;
            }
            console.log('Worker said');
            console.log(event.data);
        };
    }

    // flush message queue using request idle callbacks. retry up to 10 times.
    public flushMessageQueue(retry: number): void {
        if (!this.isReady && retry < 10) {
            setTimeout(() => {
                this.flushMessageQueue(retry + 1);
            }, 200);
            return;
        }
        if (this.isReady) {
            for (let i = 0; i < this.messageQueue.length; i += 1) {
                const message = this.messageQueue[i];
                requestIdleCallback(() => {
                    this.worker.postMessage(message);
                });
            }
            this.messageQueue = [];
        }
    }

    public getBestMove(): any {
        // add the message queue, then flush
        this.messageQueue.push({ type: 'getBestMove' });
        this.flushMessageQueue(0);
    }
}

export class MctsConnectFourGame {
    gameWrapper: GameWrapper;
    gameWorker: GameWorker;
    game: Phaser.Game;

    constructor(mctsConnectFour: typeof import('./pkg_mcts_connect_four/mcts_connect_four')) {
        this.gameWrapper = new mctsConnectFour.GameWrapper(7, 6, true);
        this.gameWorker = new GameWorker();
        this.game = new Phaser.Game({
            type: Phaser.AUTO,
            parent: 'game',
            backgroundColor: '#33A5E7',
            scale: {
                width: 800,
                height: 600,
                mode: Phaser.Scale.NO_ZOOM,
                autoCenter: Phaser.Scale.CENTER_BOTH,
            },
            scene: [new MyScene({
                width: this.gameWrapper.width(),
                height: this.gameWrapper.height(),
                squareSizePx: 50,
                squarePaddingPx: 10,
            })],
        });

        const bestMove = this.gameWorker.getBestMove();
        console.log(`bestMove: ${bestMove}`);
    }
}
