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
import { ClickDebouncer } from './pkg_mcts_connect_four';
import PauseRenderPlugin from './pause_render_plugin';

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
    GameOver,
}

enum GameStateInput {
    CpuStartThinking,
    CpuFinishedThinking,
}

class Token {
    player: ScenePlayer;
    object: Phaser.GameObjects.Ellipse;

    constructor(player: ScenePlayer, object: Phaser.GameObjects.Ellipse) {
        this.player = player;
        this.object = object;
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

    // heightPx is the total height of the board in pixels.
    heightPx: number;

    // backgroundRects is an array of Phaser.GameObjects.Rectangle that are used to draw the board. Tokens for players
    // will be drawn on top of these.
    backgroundRects: Phaser.GameObjects.Rectangle[];

    playerTokens: Token[];
    player1Color: number = 0x0000ff;
    player2color: number = 0xff0000;

    // Unnecessary fields that are inehrited but needed to make eslint happy.
    add: Phaser.GameObjects.GameObjectFactory;
    load: Phaser.Loader.LoaderPlugin;
    tweens: Phaser.Tweens.TweenManager;

    // constructor that takes a config object and populates fields.
    constructor(
        {
            width = 7,
            height = 6,
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
        this.playerTokens = [];
        this.heightPx = this.height * (this.squareSizePx + this.squarePaddingPx) + this.squarePaddingPx;

        for (let i = 0; i < this.width * this.height; i += 1) {
            this.backgroundRects.push(null);
            this.playerTokens.push(null);
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

    getPlayerToken(row: number, col: number): Token | null {
        return this.playerTokens[col * this.width + row];
    }

    setPlayerToken(row: number, col: number, object: Phaser.GameObjects.Ellipse, player: ScenePlayer) {
        this.playerTokens[col * this.width + row] = new Token(player, object);
    }

    clearPlayerToken(row: number, col: number) {
        this.playerTokens[col * this.width + row] = null;
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

        this.setPlayerToken(row, col, token, player);
    }

    // renderPop removes the bottom-most token in the column. It then moves each token above it down by one row.
    // Build up all the tween options needed and then call this.tweens.add with the options.
    renderPop(col: number) {
        this.tweens.pauseAll();
        for (let row = this.height - 1; row >= 0; row -= 1) {
            const token = this.getPlayerToken(row, col);
            if (token == null) {
                break;
            }
            let y: number;
            if (row === this.height - 1) {
                y = this.heightPx;
                this.clearPlayerToken(row, col);
                this.tweens.add({
                    targets: token.object,
                    y,
                    duration: 200,
                    ease: 'Bounce.easeOut',
                }).on('complete', () => {
                    token.object.destroy();
                });
            } else {
                const rect = this.getBackgroundRect(row + 1, col);
                y = rect.y;
                this.setPlayerToken(row + 1, col, token.object, token.player);
                this.tweens.add({
                    targets: token.object,
                    y,
                    duration: 800,
                    ease: 'Bounce.easeOut',
                });
            }
        }
        this.tweens.resumeAll();
    }

    clearMouseEventHandlers() {
        for (let i = 0; i < this.backgroundRects.length; i += 1) {
            const rect = this.backgroundRects[i];
            if (rect != null) {
                rect.setStrokeStyle(0).removeInteractive().removeAllListeners();
            }
        }
        for (let i = 0; i < this.playerTokens.length; i += 1) {
            if (this.playerTokens[i] != null) {
                this.playerTokens[i].object.removeInteractive();
            }
        }
    }

    setCellAsInteractive(
        row: number,
        column: number,
        moveType: string,
        callback: (row: number, column: number, moveType: string) => void,
    ) {
        const rect = this.getBackgroundRect(row, column);
        rect.setInteractive();

        // on mouseover, show stroke
        rect.on('pointerover', () => {
            rect.setStrokeStyle(4, 0x000000);
            console.log(`row: ${row}, column: ${column}, moveType: ${moveType}`);
        });

        rect.on('pointerout', () => {
            rect.setStrokeStyle(0);
        });

        // on mouseclick, call callback
        rect.on('pointerdown', () => {
            callback(row, column, moveType);
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

class DebugTrees {
    // data is a list of objects. The list size is the number of iterations for which MCTS ran.
    // Each object has fields (action, children, visits, wins). Children are more of the same.
    data: Array<any>;

    currentIteration: number;
    treeContainer: HTMLElement;

    constructor(
        firstButton: HTMLButtonElement,
        previousButton: HTMLButtonElement,
        nextButton: HTMLButtonElement,
        lastButton: HTMLButtonElement,
    ) {
        this.data = [];
        this.currentIteration = 0;
        this.treeContainer = document.getElementById('tree-container');
        firstButton.addEventListener('click', () => {
            this.firstIteration();
        });
        previousButton.addEventListener('click', () => {
            this.previousIteration();
        });
        nextButton.addEventListener('click', () => {
            this.nextIteration();
        });
        lastButton.addEventListener('click', () => {
            this.lastIteration();
        });
    }

    setData(data: Array<any>) {
        this.data = data;
        this.currentIteration = 0;
        this.render();
    }

    // eslint-disable-next-line class-methods-use-this
    render() {
        const currentTree = this.data[this.currentIteration];
        const json = JSON.stringify(currentTree, null, 2);
        this.treeContainer.innerHTML = json;

        // const currentTree = this.data[this.currentIteration];
        // const svg = d3.select('#tree-container').append('svg');
        // const treemap = d3.tree().size([100, 100]);
        // const root = d3.hierarchy(currentTree, (d) => d.children);
    }

    nextIteration() {
        const maxIterations = this.data.length;
        this.currentIteration += 1;
        if (this.currentIteration >= maxIterations) {
            this.currentIteration = maxIterations - 1;
        }
        this.render();
    }

    previousIteration() {
        this.currentIteration -= 1;
        if (this.currentIteration < 0) {
            this.currentIteration = 0;
        }
        this.render();
    }

    lastIteration() {
        this.currentIteration = this.data.length - 1;
        this.render();
    }

    firstIteration() {
        this.currentIteration = 0;
        this.render();
    }
}

export class MctsConnectFourGame {
    cpuThinking: HTMLElement;
    gameStatus: HTMLElement;
    gameState: GameState;
    debugTrees: DebugTrees;
    firstButton: HTMLButtonElement;
    previousButton: HTMLButtonElement;
    nextButton: HTMLButtonElement;
    lastButton: HTMLButtonElement;

    gameWorker: GameWorker;
    scene: MyScene;
    game: Phaser.Game;

    clickDebouncer: ClickDebouncer;

    constructor() {
        this.cpuThinking = document.getElementById('cpu-thinking');
        this.hideCpuIsThinking();

        this.gameStatus = document.getElementById('game-status');
        this.hideGameStatus();

        this.firstButton = document.getElementById('first-button') as HTMLButtonElement;
        this.previousButton = document.getElementById('previous-button') as HTMLButtonElement;
        this.nextButton = document.getElementById('next-button') as HTMLButtonElement;
        this.lastButton = document.getElementById('last-button') as HTMLButtonElement;
        this.debugTrees = new DebugTrees(this.firstButton, this.previousButton, this.nextButton, this.lastButton);

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
                autoCenter: Phaser.Scale.CENTER_HORIZONTALLY,
            },
            plugins: {
                global: [
                    { key: 'PhaserPauseRenderPlugin', plugin: PauseRenderPlugin, mapping: 'render' },
                ],
            },
            scene: [this.scene],
        });

        this.gameState = GameState.CpuTurn;

        import('./pkg_mcts_connect_four/mcts_connect_four').then((module) => {
            this.clickDebouncer = new module.ClickDebouncer();
            this.changeState(GameStateInput.CpuStartThinking);
        });
    }

    showCpuIsThinking(): void {
        this.cpuThinking.classList.remove('invisible');
        this.cpuThinking.classList.add('visible');
    }

    hideCpuIsThinking(): void {
        this.cpuThinking.classList.remove('visible');
        this.cpuThinking.classList.add('invisible');
    }

    showGameStatus(text: string): void {
        this.gameStatus.classList.remove('invisible');
        this.gameStatus.classList.add('visible');
        this.gameStatus.innerText = text;
    }

    hideGameStatus(): void {
        this.gameStatus.classList.remove('visible');
        this.gameStatus.classList.add('invisible');
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
                this.clickDebouncer.clear();
                this.gameState = GameState.CpuTurn;
                this.showCpuIsThinking();
                const bestMove = await this.gameWorker.getBestMove();
                console.log('bestMove');
                console.log(bestMove);
                this.debugTrees.setData(bestMove.debug_trees);

                const moveType = bestMove.actual_move.move_type;
                const col = bestMove.actual_move.column;
                const newState = await this.gameWorker.applyMove(moveType, col);
                console.log('newState');
                console.log(newState);

                if (moveType === 'Insert') {
                    const row = bestMove.maybe_insert_row;
                    this.scene.renderInsert(ScenePlayer.Player1, row, col);
                } else {
                    this.scene.renderPop(col);
                }

                if (newState.is_terminal_position !== 'IsNotTerminal') {
                    this.gameState = GameState.GameOver;
                    this.hideCpuIsThinking();
                    if (newState.is_terminal_position.IsTerminalWin === 'Player1') {
                        this.showGameStatus('Player 1 won!');
                    } else if (newState.is_terminal_position.IsTerminalWin === 'Player2') {
                        this.showGameStatus('Player 2 won!');
                    } else {
                        this.showGameStatus('Draw!');
                    }
                    return;
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
                    console.log('setting up legal moves');
                    console.log(move);
                    this.scene.setCellAsInteractive(
                        move.row,
                        move.column,
                        move.move_type,
                        async (row: number, column: number, moveType: string) => {
                            if (this.clickDebouncer.is_present(row, column)) {
                                console.log('debounce');
                                return;
                            }
                            this.clickDebouncer.add(row, column);
                            console.log('Human clicked on cell');
                            console.log(moveType);
                            console.log(row);
                            console.log(column);

                            const newState = await this.gameWorker.applyMove(moveType, column);
                            console.log('newState');
                            console.log(newState);

                            if (moveType === 'Insert') {
                                this.scene.renderInsert(ScenePlayer.Player2, row, column);
                            } else {
                                this.scene.renderPop(column);
                            }

                            if (newState.is_terminal_position !== 'IsNotTerminal') {
                                this.gameState = GameState.GameOver;
                                this.hideCpuIsThinking();
                                if (newState.is_terminal_position.IsTerminalWin === 'Player1') {
                                    this.showGameStatus('Player 1 won!');
                                } else if (newState.is_terminal_position.IsTerminalWin === 'Player2') {
                                    this.showGameStatus('Player 2 won!');
                                } else {
                                    this.showGameStatus('Draw!');
                                }
                                return;
                            }

                            this.gameState = GameState.CpuTurn;
                            this.waitForCpuTurn();
                        },
                    );
                }
            }
            break;

        default:
            console.log('Unhandled state transition');
        }
    }
}
