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

import('./pkg_mcts_connect_four/mcts_connect_four').then((mctsConnectFour) => {
    const gameWrapper = new mctsConnectFour.GameWrapper(7, 6, true);

    // eslint-disable-next-line no-inner-declarations
    function onMessage(event) {
        const { data } = event;
        if (data.type === 'getBestMove') {
            const bestMove = gameWrapper.get_mcts_best_move();
            postMessage(bestMove);
        } else if (data.type === 'applyMove') {
            const result = gameWrapper.apply_move(data.move);
            postMessage(result);
        } else if (data.type === 'getLegalMovesCells') {
            const legalMoves = gameWrapper.get_legal_moves_cells();
            postMessage(legalMoves);
        } else if (data.type === 'turn') {
            const turn = gameWrapper.turn();
            postMessage(turn);
        }
    }

    // eslint-disable-next-line no-restricted-globals
    addEventListener('message', onMessage);

    postMessage({ type: 'ready' });
});
