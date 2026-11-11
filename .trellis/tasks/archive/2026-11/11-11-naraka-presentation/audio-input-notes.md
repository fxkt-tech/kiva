# Audio/input implementation boundary

用户已批准父方案和全部子任务。此实现切片只拥有client/audio.rs、client/controls.rs、web/audio.js、audio-unit测试及必要同目录专用测试；动作worker继续拥有source/animation/avatar/viewer/companion/mod，资产worker拥有冻结媒体和config。不开服务、不提交/推送、不改canonical spec。

当前差距：Rust把所有挥击/防御合并到旧三cue，JS只加载单文件effect，没有原变体/分层/优先级；controls失活clear已通过新增模拟语义锁存取消，需要真实sample回归防止后继恢复。新层只消费权威反馈/快照，不调模拟规则。

实现：Rust普通挥击按MeleeStart的不可变身份和当前同ActionSnapshot.elapsed调度，旧player Melee marker不重复播；真实defender决定新防御声，MotionFeedback独立游标。JS读取有界version1白名单目录、权重随机和variant gain，落地两个PlayAction按AudioBufferSource.start分层延时，每个节点计4声部，低优先级不挤占关键声，暂停/替换context取消未来节点。

保留旧枪声/BGM/燕云与E/Q回退；不增加查看页音频或音量面板，不改已冻结文件。
