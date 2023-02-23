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
    preload() {
        this.load.image('logo', 'favicon-32x32.png');
    }

    create() {
        const logo = this.add.image(400, 70, 'logo');
        this.tweens.add({
            targets: logo,
            y: 500,
            duration: 2000,
            ease: 'Power2',
            yoyo: true,
            loop: -1,

        });
    }
}

export class MctsConnectFourGame {
    gameWrapper: GameWrapper;
    game: Phaser.Game;

    constructor(mctsConnectFour: typeof import('./pkg_mcts_connect_four/mcts_connect_four')) {
        this.gameWrapper = new mctsConnectFour.GameWrapper(7, 6, false);
        this.game = new Phaser.Game({
            type: Phaser.AUTO,
            parent: 'game',
            backgroundColor: '#33A5E7',
            scale: {
                width: 800,
                height: 600,
                mode: Phaser.Scale.FIT,
                autoCenter: Phaser.Scale.CENTER_BOTH,
            },
            scene: [new MyScene()],
        });
    }
}
