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

const importPromise = import('./pkg_mcts_connect_four/mcts_connect_four');

export default function importPromiseMctsConnectFour() {
    return importPromise;
}

enum ScenePlayer {
    Player1,
    Player2
}

enum GameState {
    CpuTurn,
    HumanTurn,
}

enum GameStateInput {
    CpuStartThinking,
    CpuFinishedThinking,
}

class Cell {
    row: number;
    column: number;

    constructor(row: number, column: number) {
        this.row = row;
        this.column = column;
    }
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

    player1Tokens: Phaser.GameObjects.Ellipse[];
    player1Color: number = 0x0000ff;
    player2Tokens: Phaser.GameObjects.Ellipse[];
    player2color: number = 0xff0000;

    // Unnecessary fields that are inehrited but needed to make eslint happy.
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
        this.player1Tokens = [];
        this.player2Tokens = [];

        for (let i = 0; i < this.width * this.height; i += 1) {
            this.backgroundRects.push(null);
            this.player1Tokens.push(null);
            this.player2Tokens.push(null);
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
    }

    getBackgroundRect(row: number, col: number): Phaser.GameObjects.Rectangle {
        return this.backgroundRects[col * this.width + row];
    }

    getPlayer1Token(row: number, col: number): Phaser.GameObjects.Ellipse {
        return this.player1Tokens[col * this.width + row];
    }

    setPlayer1Token(row: number, col: number, object: Phaser.GameObjects.Ellipse) {
        this.player1Tokens[col * this.width + row] = object;
    }

    getPlayer2Token(row: number, col: number): Phaser.GameObjects.Ellipse {
        return this.player2Tokens[col * this.width + row];
    }

    setPlayer2Token(row: number, col: number, object: Phaser.GameObjects.Ellipse) {
        this.player2Tokens[col * this.width + row] = object;
    }

    renderInsert(player: ScenePlayer, row: number, col: number) {
        const rect = this.getBackgroundRect(row, col);

        // token starts above the column above the top row and then tweens down to the rect.x rect.y location.
        const token = this.add.circle(
            rect.x,
            0,
            rect.width / 2,
            player === ScenePlayer.Player1 ? this.player1Color : this.player2color,
        );
        this.tweens.add({
            targets: token,
            y: rect.y,
            duration: 800,
            ease: 'Bounce.easeOut',
        });

        if (player === ScenePlayer.Player1) {
            this.setPlayer1Token(row, col, token);
        } else if (player === ScenePlayer.Player2) {
            this.setPlayer2Token(row, col, token);
        }
    }

    clearMouseEventHandlers() {
        for (let i = 0; i < this.backgroundRects.length; i += 1) {
            if (this.backgroundRects[i] !== null) {
                this.backgroundRects[i].setStrokeStyle(0);
                this.backgroundRects[i].removeInteractive();
            }
        }
        for (let i = 0; i < this.player1Tokens.length; i += 1) {
            if (this.player1Tokens[i] !== null) {
                this.player1Tokens[i].removeInteractive();
            }
        }
        for (let i = 0; i < this.player2Tokens.length; i += 1) {
            if (this.player2Tokens[i] !== null) {
                this.player2Tokens[i].removeInteractive();
            }
        }
    }

    setCellAsInteractive(row: number, column: number, callback: () => void) {
        const rect = this.getBackgroundRect(row, column);
        rect.setInteractive();

        // on mouseover, show stroke
        rect.on('pointerover', () => {
            rect.setStrokeStyle(4, 0x000000);
        });

        rect.on('pointerout', () => {
            rect.setStrokeStyle(0);
        });

        // on mouseclick, call callback
        rect.on('pointerdown', () => {
            callback();
        });
    }
}

class MessageAndCallback {
    message: any;
    callback: (data: any) => void;

    constructor(message: any, callback: (data: any) => void) {
        this.message = message;
        this.callback = callback;
    }
}

class GameWorker {
    worker: Worker;
    isReady: boolean;
    outgoingMessageQueue: MessageAndCallback[];
    incomingMessageQueue: MessageAndCallback[];

    // eslint-disable-next-line no-unused-vars
    currentWorkerCallback: (data: any) => void;

    constructor() {
        this.isReady = false;
        this.outgoingMessageQueue = [];
        this.incomingMessageQueue = [];

        // @ts-ignore
        this.worker = new Worker(new URL('./mcts_connect_four_worker.ts', import.meta.url));

        this.worker.onmessage = (event) => {
            if (event.data.type === 'ready') {
                console.log('Worker is ready');
                this.isReady = true;
                return;
            }

            if (this.incomingMessageQueue.length > 0) {
                const messageAndCallback = this.incomingMessageQueue.shift();
                console.log('incoming message');
                console.log(event.data);
                messageAndCallback.callback(event.data);
            }
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
            while (this.outgoingMessageQueue.length > 0) {
                const messageAndCallback = this.outgoingMessageQueue.shift();
                console.log('outgoing message');
                console.log(messageAndCallback.message);
                this.worker.postMessage(messageAndCallback.message);
                this.incomingMessageQueue.push(messageAndCallback);
            }

            if (this.incomingMessageQueue.length > 0) {
                if (retry !== -1) {
                    setTimeout(() => {
                        this.flushMessageQueue(retry + 1);
                    }, 200);
                }
            }
        }
    }

    async requestResponse(message: any): Promise<any> {
        return new Promise((resolve) => {
            const messageAndCallback = new MessageAndCallback(message, resolve);
            this.outgoingMessageQueue.push(messageAndCallback);
            this.flushMessageQueue(0);
        });
    }

    public async getBestMove(): Promise<any> {
        return this.requestResponse({ type: 'getBestMove' });
    }

    public async applyMove(moveType: string, column: number): Promise<any> {
        return this.requestResponse({
            type: 'applyMove',
            move: {
                move_type: moveType,
                column,
            },
        });
    }

    public async getLegalMovesCells(): Promise<any> {
        return this.requestResponse({ type: 'getLegalMovesCells' });
    }

    public async turn(): Promise<any> {
        return this.requestResponse({ type: 'turn' });
    }
}

export class MctsConnectFourGame {
    cpuThinking: HTMLElement;
    gameState: GameState;

    gameWorker: GameWorker;
    scene: MyScene;
    game: Phaser.Game;

    cellEventBlocklist: Set<Cell>;

    constructor() {
        this.cpuThinking = document.getElementById('cpu-thinking');
        this.gameWorker = new GameWorker();
        this.scene = new MyScene({
            width: 7,
            height: 6,
            squareSizePx: 50,
            squarePaddingPx: 10,
        });
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
            scene: [this.scene],
        });

        this.gameState = GameState.CpuTurn;
        this.cellEventBlocklist = new Set();
        this.changeState(GameStateInput.CpuStartThinking);
    }

    showCpuIsThinking(): void {
        this.cpuThinking.classList.remove('invisible');
        this.cpuThinking.classList.add('visible');
    }

    hideCpuIsThinking(): void {
        this.cpuThinking.classList.remove('visible');
        this.cpuThinking.classList.add('invisible');
    }

    async waitForCpuTurn(): Promise<void> {
        console.log('waitForCpuTurn');
        this.gameWorker.flushMessageQueue(-1);
        const turn = await this.gameWorker.turn();
        if (turn === 'Player1') {
            await this.changeState(GameStateInput.CpuStartThinking);
        } else {
            setTimeout(() => {
                this.waitForCpuTurn();
            }, 500);
        }
    }

    async changeState(gameStateInput: GameStateInput) {
        switch (this.gameState) {
        case GameState.CpuTurn:
            if (gameStateInput === GameStateInput.CpuStartThinking) {
                this.scene.clearMouseEventHandlers();
                this.cellEventBlocklist.clear();
                this.gameState = GameState.CpuTurn;
                this.showCpuIsThinking();
                const bestMove = await this.gameWorker.getBestMove();
                console.log('bestMove');
                console.log(bestMove);

                const moveType = bestMove.actual_move.move_type;
                const col = bestMove.actual_move.column;
                await this.gameWorker.applyMove(moveType, col);

                if (moveType === 'Insert') {
                    const row = bestMove.maybe_insert_row;
                    this.scene.renderInsert(ScenePlayer.Player1, row, col);
                }

                await this.changeState(GameStateInput.CpuFinishedThinking);
            } else if (gameStateInput === GameStateInput.CpuFinishedThinking) {
                this.hideCpuIsThinking();
                this.gameState = GameState.HumanTurn;
                const legalMoves = await this.gameWorker.getLegalMovesCells();

                // legalMoves has array of moves. each move is an object with move_type, row, and column.
                // for each move, call setCellAsInteractive.
                for (let i = 0; i < legalMoves.moves.length; i += 1) {
                    const move = legalMoves.moves[i];
                    this.scene.setCellAsInteractive(move.row, move.column, async () => {
                        console.log(this.cellEventBlocklist);
                        const cell = new Cell(move.row, move.column);
                        if (this.cellEventBlocklist.has(cell)) {
                            console.log('debounce');
                            return;
                        }
                        this.cellEventBlocklist.add(cell);
                        console.log('Human clicked on cell');
                        console.log(move.row);
                        console.log(move.column);

                        await this.gameWorker.applyMove(move.move_type, move.column);
                        if (move.move_type === 'Insert') {
                            this.scene.renderInsert(ScenePlayer.Player2, move.row, move.column);
                        }
                        this.gameState = GameState.CpuTurn;
                        this.waitForCpuTurn();
                    });
                }
            }
            break;

        default:
            console.log('Unhandled state transition');
        }
    }
}
