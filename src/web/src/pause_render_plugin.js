/*
 * From https://github.com/samme/phaser-plugin-pause-render
 * SPDX-License-Identifier: MIT
 */

import Phaser from 'phaser';

const {
    PRE_STEP, STEP, POST_STEP, PRE_RENDER, POST_RENDER, DESTROY,
} = Phaser.Core.Events;

const origStep = Phaser.Game.prototype.step;
let paused = false;

// eslint-disable-next-line consistent-return,func-names
const step = function (time, delta) {
    if (this.pendingDestroy) {
        return this.runDestroy();
    }

    const { events, renderer } = this;

    events.emit(PRE_STEP, time, delta);
    events.emit(STEP, time, delta);
    this.scene.update(time, delta);
    events.emit(POST_STEP, time, delta);

    // eslint-disable-next-line consistent-return
    if (paused) return;

    renderer.preRender();
    events.emit(PRE_RENDER, renderer, time, delta);
    this.scene.render(renderer);
    renderer.postRender();
    events.emit(POST_RENDER, renderer, time, delta);
};

export default class PauseRenderPlugin extends Phaser.Plugins.BasePlugin {
    init(data) {
        this.paused = data ? data.paused : false;
    }

    start() {
        this.game.events.once(DESTROY, this.destroy, this);
        this.game.step = step;
    }

    stop() {
        this.resume();
        this.game.step = origStep;
    }

    destroy() {
        this.stop();
        super.destroy();
    }

    // eslint-disable-next-line class-methods-use-this
    pause() {
        paused = true;
    }

    // eslint-disable-next-line class-methods-use-this
    resume() {
        paused = false;
    }

    // eslint-disable-next-line class-methods-use-this
    get paused() {
        return paused;
    }

    // eslint-disable-next-line class-methods-use-this
    set paused(val) {
        paused = val;
    }
}
