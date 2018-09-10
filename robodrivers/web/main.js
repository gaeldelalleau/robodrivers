var Robodrivers = new Phaser.Class({

    Extends: Phaser.Scene,

    initialize: function Robodrivers()
    {
        Phaser.Scene.call(this, { key: 'robodrivers' });
        this.connect();

        this.cars;
        this.scoreTexts;
    },

    game_state: null,
    updated: false,
    map_created: false,

    config:
    {
        block_size: 32,
    },

    preload: function ()
    {
        //this.load.setBaseURL('http://labs.phaser.io');

        this.load.image('car', 'assets/car.jpg');
        this.load.image('block', 'assets/block.jpg');
        this.load.image('sky', 'assets/space3.png');
        this.load.image('logo', 'assets/phaser3-logo.png');
        this.load.image('red', 'assets/red.png');
        /*
         this.load.spritesheet('dude',
            'assets/dude.png',
            { frameWidth: 32, frameHeight: 48 }
        );
        */
    },

    create: function ()
    {

        this.add.image(400, 300, 'sky');

        /*
        var particles = this.add.particles('red');
        var emitter = particles.createEmitter({
            speed: 100,
            scale: { start: 1, end: 0 },
            blendMode: 'ADD'
        });
        */


        /*
        platforms = this.physics.add.staticGroup();
        platforms.create(400, 568, 'ground').setScale(2).refreshBody();

        player = this.physics.add.sprite(100, 450, 'dude');
        player.setBounce(0.2);
        player.setCollideWorldBounds(true);

        this.physics.add.collider(player, platforms);

        this.anims.create({
            key: 'left',
            frames: this.anims.generateFrameNumbers('dude', { start: 0, end: 3 }),
            frameRate: 10,
            repeat: -1
        });
        */

        this.cars = [];
        for (i=0; i<8; i++)
        {
            var car = this.physics.add.sprite(0, 0, 'car');
            car.displayWidth = this.config.block_size;
            car.displayHeight = this.config.block_size;
            car.setBounce(0.2);
            car.setCollideWorldBounds(true);
            this.cars.push(car);
        }

        this.scoreTexts = [];
        for (i=0; i<8; i++)
        {
            this.scoreTexts.push(this.add.text(16, 16 + i*20, '', { fontSize: '16px', fill: '#00ff00' }));
        }

        /*
        var logo = this.physics.add.image(400, 100, 'logo');
        logo.setVelocity(100, 200);
        logo.setBounce(1, 1);
        logo.setCollideWorldBounds(true);

        emitter.startFollow(logo);
        */

    },

    health_to_bar: function(health, max_health)
    {
        var bar = "";
        for (var i=0; i<max_health; i++)
        {
            bar += (i<health) ? "*" : " ";
        }
        return bar;
    },

    create_map: function(map) {
        for (y=0; y < map.cells.length; y++) {
            row = map.cells[y];
            for (x=0; x<row.length; x++) {
                cell = row[x];
                this.add.image(x*this.config.size, y*this.config.size, 'block');
            }
        }
    },

    update: function()
    {
        if (this.updated === true)
        {
            this.updated = false;
            //console.log(game_state);

            if (this.map_created === false) {
                this.map_created = true;
                this.create_map(this.game_state.map);
            }

            Object.keys(this.game_state.teams).forEach(id =>
            {
                var team = this.game_state.teams[id];
                var car = this.game_state.cars[id];
                var car_sprite = this.cars[id-1];  // TODO: fix that by using correct dict for sprite cars, built from the real team ids
                car_sprite.setTint(parseInt(team.color.replace(/^#/, ''), 16));
                var x = this.config.block_size*car.x;
                var y = this.config.block_size*car.y;
                car_sprite.setPosition(x, y);
                car_sprite.setAngle(0);

                this.scoreTexts[id-1].setText(this.game_state.teams[id].name + ': ' +
                    this.health_to_bar(car.killed ? 0 : car.health, car.max_health) +
                    ' ' + this.game_state.teams[id].score); // TODO: fix that by using correct dict for sprite cars, built from the real team ids
            });

        }

        // player.setVelocityX(-160);
       // player.anims.play('left', true);
    },


    connect: function()
    {
        var url = "ws://" + location.hostname + ":3012";
        socket = new ReconnectingWebSocket(url);
        socket.onmessage = function(event)
        {
            var msg = JSON.parse(event.data);
            var time = new Date(msg.date);
            //console.log(msg);
            this.game_state = msg;
            this.updated = true;
        }.bind(this);

    },
});

var phaser_config = {
    type: Phaser.AUTO,
    width: 800,
    height: 680,
    backgroundColor: '#000000',
    physics: {
        default: 'arcade',
        arcade: {
            gravity: { y: 0 }
        }
    },
    scene: [ Robodrivers ],
}

var game = new Phaser.Game(phaser_config);
