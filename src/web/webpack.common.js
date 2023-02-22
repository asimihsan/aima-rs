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

const path = require('path');

const CopyPlugin = require("copy-webpack-plugin");
const CssMinimizerPlugin = require('css-minimizer-webpack-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

module.exports = {
    entry: {
        index: './src/index.ts',
    },
    resolve: {
        extensions: ['.ts', '.tsx', '.js', '.jsx', '.mjs', '.json', '.wasm'],
    },
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: '[name]/[name].[contenthash].js',
        publicPath: 'auto',
        clean: false,
    },

    // See: https://webpack.js.org/guides/caching/
    optimization: {
        moduleIds: 'deterministic',
        runtimeChunk: 'single',
        splitChunks: {
            cacheGroups: {
                vendor: {
                    test: /[\\/]node_modules[\\/]/,
                    name: 'vendors',
                    chunks: 'all',
                }
            }
        },
        minimize: true,
        minimizer: [
            `...`,
            new CssMinimizerPlugin(),
        ],
    },
    module: {
        rules: [
            {
                test: /\.(mjs|js|jsx|ts|tsx)$/,
                exclude: /node_modules/,
                use: {
                    loader: 'babel-loader',
                    options: {
                        presets: [
                            '@babel/preset-env',
                        ],
                        plugins: [
                            ["@babel/transform-runtime", {
                                "regenerator": true,
                            }],
                        ]
                    }
                }
            },
            {
                test: /\.css$/,
                use: [MiniCssExtractPlugin.loader, 'css-loader'],
            },
            {
                test: /\.(png|svg|jpg|jpeg|gif)$/i,
                type: 'asset/resource',
            },
            {
                test: /\.(woff|woff2|eot|ttf|otf)$/i,
                type: 'asset/resource',
            },
            {
                test: /\.wasm$/,
                type: 'webassembly/sync',
            }
        ]
    },
    experiments: {
        syncWebAssembly: true,
    },
    plugins: [
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, '..', 'wasm', 'wasm-bindgen-mcts-connect-four'),
            watchDirectories: [
                path.resolve(__dirname, '..', 'wasm', 'wasm-bindgen-mcts-connect-four'),
                path.resolve(__dirname, '..', 'lib', 'game', 'connect-four-logic'),
                path.resolve(__dirname, '..', 'lib', 'end-to-end', 'mcts-connect-four'),
            ],
            outDir: path.resolve(__dirname, 'src', 'pkg_mcts_connect_four'),
            outName: "mcts_connect_four",
            forceMode: "production",
        }),
        new MiniCssExtractPlugin({
            filename: '[name].[contenthash].css',
            chunkFilename: '[id].[contenthash].css',
        }),
        new CopyPlugin({
            patterns: [
                path.resolve(__dirname, 'src', 'favicon.ico'),
                path.resolve(__dirname, 'src', 'android-chrome-192x192.png'),
                path.resolve(__dirname, 'src', 'android-chrome-512x512.png'),
                path.resolve(__dirname, 'src', 'apple-touch-icon.png'),
                path.resolve(__dirname, 'src', 'favicon-16x16.png'),
                path.resolve(__dirname, 'src', 'favicon-32x32.png'),
            ],
        }),
        new HtmlWebpackPlugin({
            title: 'Rust WASM Webpack Example',
            template: path.resolve(__dirname, 'src', 'index.html'),
            filename: 'index.html',
            chunks: ['index'],
        })
    ]
};
