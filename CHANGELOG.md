# Changelog

## [0.3.0](https://github.com/Morfusee/subtracker-cli/compare/subtracker-v0.2.0...subtracker-v0.3.0) (2026-08-29)


### Features

* add antigravity quota provider ([a002246](https://github.com/Morfusee/subtracker-cli/commit/a002246d766a6159b0b3b6b05c7388e46d259058))
* add horizontal padding to container cards ([04da28d](https://github.com/Morfusee/subtracker-cli/commit/04da28d4e809ec271aaf8a0922d80b86e3f1efe2))
* add input and terminal lifecycle ([18a2f69](https://github.com/Morfusee/subtracker-cli/commit/18a2f695d66bab92f657e4df75cdddc661f2d1bb))
* add opencode usage provider ([6197340](https://github.com/Morfusee/subtracker-cli/commit/6197340957b1bd0baa2fa8cab4fc50747c070a13))
* add provider process boundary ([3b98214](https://github.com/Morfusee/subtracker-cli/commit/3b982142366adebfec67e8feeeda19220836089f))
* add refresh state machine ([a3efa91](https://github.com/Morfusee/subtracker-cli/commit/a3efa9159bcf1386b148131edcb045625c1dd03a))
* add Vim-style STC ASCII banner header centered above dashboard ([4eb501f](https://github.com/Morfusee/subtracker-cli/commit/4eb501f55f470dfc4d5c56b7073caa30fa812c0a))
* **app:** track update modal state ([15d6695](https://github.com/Morfusee/subtracker-cli/commit/15d6695341efc29f6bac35c14d3dfb8d680a4c6d))
* bootstrap subtracker data model ([1ad4816](https://github.com/Morfusee/subtracker-cli/commit/1ad481631e8d37938cbbbedc80187d0cb0de307f))
* center dashboard layout horizontally and vertically ([0425890](https://github.com/Morfusee/subtracker-cli/commit/0425890669d8393845c1ff36df9a50ffea4f002a))
* **cli:** schedule release update checks ([96396a3](https://github.com/Morfusee/subtracker-cli/commit/96396a3716c49ab1fb6cb24290e00566b1e2eb99))
* fetch current codex quota ([1a52abf](https://github.com/Morfusee/subtracker-cli/commit/1a52abf38c8339ad73dd7df4a0317dbf09f65edb))
* implement visual redesign with theme, segmented quota bars, and responsive layouts ([0b27462](https://github.com/Morfusee/subtracker-cli/commit/0b27462b8be33ca1cd684c2ecc59df9a9c114339))
* increase subscription text indentation to 7 characters ([fcc55f6](https://github.com/Morfusee/subtracker-cli/commit/fcc55f6d391def4a852e29a639203d25ebc5e0df))
* indent service titles on card border ([c623582](https://github.com/Morfusee/subtracker-cli/commit/c623582f7a12d033a0d305b58ab3012bf8302a8c))
* parse codex usage data ([8d2464b](https://github.com/Morfusee/subtracker-cli/commit/8d2464b4d63eb31a05e11b6b24604de178b45887))
* refresh providers concurrently ([085ac04](https://github.com/Morfusee/subtracker-cli/commit/085ac04a210e0d7f87bd86065d32cc55a7691182))
* remove 'remaining' in wide mode and center details in compact mode ([a07e4cb](https://github.com/Morfusee/subtracker-cli/commit/a07e4cb024a6635f14370f4b8c8eaa07cdc86db9))
* render status line on bottom-right border of card blocks ([4a6988a](https://github.com/Morfusee/subtracker-cli/commit/4a6988a0bf43e7b44749a99cbc101aaa85c133cd))
* render subtracker dashboard ([ad7bf28](https://github.com/Morfusee/subtracker-cli/commit/ad7bf283015c4d624143ef3bef06373f325549c6))
* **runtime:** route update modal actions ([d0f5f44](https://github.com/Morfusee/subtracker-cli/commit/d0f5f443e83b3334396b3d753a74d4787ded4e04))
* support opencode-go and account.json auth formats ([1e516c7](https://github.com/Morfusee/subtracker-cli/commit/1e516c79d157dd5cdf6718203ad94920e37790b7))
* switch opencode provider to go subscription usage endpoint ([b96741d](https://github.com/Morfusee/subtracker-cli/commit/b96741d46aabd0d7da276bbc9cbbd0973eec0f8a))
* switch to dark pastel palette and indent subscription rows ([09c4cd3](https://github.com/Morfusee/subtracker-cli/commit/09c4cd30de0a72f9891c6dc7c5e87538faaf75e6))
* **ui:** add release update spotlight ([69d3414](https://github.com/Morfusee/subtracker-cli/commit/69d3414bbd623a44a32c34064c66c6208aa752fb))
* **ui:** add responsive density scaling, logo rules, and quota spacing ([f3ba65f](https://github.com/Morfusee/subtracker-cli/commit/f3ba65fef7eb5d9522cbaf2d88a0ad00d0777594))
* **ui:** add update preview mode ([56e1172](https://github.com/Morfusee/subtracker-cli/commit/56e11720562d0d700f3af886de64221aa33030e0))
* **ui:** grid layout, bar expansion and phase docs ([f06bda3](https://github.com/Morfusee/subtracker-cli/commit/f06bda3f8095966182af8a8f01bf8f86685428f7))
* **ui:** map keyboard controls to provider cards ([077219f](https://github.com/Morfusee/subtracker-cli/commit/077219f5b12d19b016e2127ff96632e9afe337de))
* **ui:** preserve status on collapsed provider cards ([cf632fc](https://github.com/Morfusee/subtracker-cli/commit/cf632fcbcdb6694967f3eaa92586c66d50e6dd62))
* **ui:** render focused collapsible provider cards ([b8d1925](https://github.com/Morfusee/subtracker-cli/commit/b8d19256558267cbd28c7f0004e48197bb2ec007))
* **ui:** track provider focus and collapse state ([811b870](https://github.com/Morfusee/subtracker-cli/commit/811b8704144286804375f9dabc17a7cb21f7c00b))
* **updater:** check GitHub releases ([ce2cb4c](https://github.com/Morfusee/subtracker-cli/commit/ce2cb4cb34c45d61509e84918537e46b8fe46b11))
* use textured dark shade glyphs for filled quota bars ([0a245c9](https://github.com/Morfusee/subtracker-cli/commit/0a245c9d3fdabccc77c05f2c90ff3f6fe41ab980))
* wire subtracker runtime ([69bc5cd](https://github.com/Morfusee/subtracker-cli/commit/69bc5cd27cec10a4be99f96f3f7d3db9cb80c19c))


### Bug Fixes

* add top and bottom interior padding inside service boxes across all modes ([f7bba24](https://github.com/Morfusee/subtracker-cli/commit/f7bba24e206040da9e152888f6767fc39cf592bc))
* ensure balanced top and bottom padding across large and small terminals ([8d1f32e](https://github.com/Morfusee/subtracker-cli/commit/8d1f32eca2c14c38f88a86f2eb05e89aa0aa6062))
* position footer directly below cards and style keycaps ([deb47e1](https://github.com/Morfusee/subtracker-cli/commit/deb47e1b37e1a1f353b15ed10a8d402dcb76078d))
* **ui:** bundle card render parameters to resolve clippy too_many_arguments ([b9ac659](https://github.com/Morfusee/subtracker-cli/commit/b9ac65939c7a639bd6a0eca9dda8df81c5e64afa))
* **updater:** handle browser opener failures ([a488482](https://github.com/Morfusee/subtracker-cli/commit/a4884821980771d1dde5bd3ad1dab6f45976d43e))
