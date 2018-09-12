var Robodrivers = new Phaser.Class({

    Extends: Phaser.Scene,

    initialize: function Robodrivers()
    {
        Phaser.Scene.call(this, { key: 'robodrivers' });
        this.connect();

        this.background;
        this.cars;
        this.scoreTexts;
    },

    game_state: null,
    updated: false,
    map_created: false,
    transient_sprites: [],

    config:
    {
        block_size: 32,
    },

    preload: function ()
    {
        //this.load.setBaseURL('http://labs.phaser.io');

        this.load.image('background', 'assets/icon-logo.png');
        this.load.image('car', 'assets/car.jpg');
        this.load.image('block', 'assets/block.jpg');
        this.load.image('base', 'assets/base.png');
        this.load.image('resource', 'assets/resource.png');
        this.load.image('producer', 'assets/producer.png');
        this.load.image('explosion', 'assets/explosion.png');
        /*
         this.load.spritesheet('dude',
            'assets/dude.png',
            { frameWidth: 32, frameHeight: 48 }
        );
        */
    },

    create: function ()
    {

        this.background = this.add.image(800, 320, 'background');

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

        /*
        var logo = this.physics.add.image(400, 100, 'logo');
        logo.setVelocity(100, 200);
        logo.setBounce(1, 1);
        logo.setCollideWorldBounds(true);

        emitter.startFollow(logo);
        */
    },

    create_cars: function(teams)
    {
        this.cars = [];
        Object.keys(teams).forEach(team_id =>
        {
            var team = teams[team_id];
            var car = this.physics.add.sprite(-100000, -100000, 'car');
            car.depth = 1;
            car.setDisplaySize(this.config.block_size, this.config.block_size);
            car.setBounce(0.2);
            car.setCollideWorldBounds(true);
            this.cars[team_id] = car;
        });
    },

    create_ui: function(teams)
    {
        this.scoreTexts = [];
        var i = 0;
        Object.keys(teams).forEach(team_id =>
        {
            var team = teams[team_id];
            var scoreText = this.add.text(16, 16 + i*20, '', { fontSize: '16px', fill: team.color });
            scoreText.depth = 2;
            this.scoreTexts[team_id] = scoreText;
            i++;
        });
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

    create_map: function(map)
    {
        for (y=0; y < map.cells.length; y++)
        {
            row = map.cells[y];
            for (x=0; x<row.length; x++)
            {
                cell = row[x];
                if (cell.block == "WALL")
                {
                    var block = this.add.sprite(x*this.config.block_size, y*this.config.block_size, 'block');
                    block.setDisplaySize(this.config.block_size, this.config.block_size);
                }
                cell.items.forEach(function(item) {
                    if (item === "BASE")
                    {
                        var item = this.add.sprite(x*this.config.block_size, y*this.config.block_size, 'base');
                        item.setDisplaySize(this.config.block_size, this.config.block_size);
                    }
                    else if (item.hasOwnProperty("PRODUCER"))
                    {
                        var item = this.add.sprite(x*this.config.block_size, y*this.config.block_size, 'producer');
                        item.setDisplaySize(this.config.block_size, this.config.block_size);
                    }
                }.bind(this));
            }
        }
    },

    parse_map: function(map)
    {
        for (y=0; y < map.cells.length; y++)
        {
            row = map.cells[y];
            for (x=0; x<row.length; x++)
            {
                cell = row[x];
                cell.items.forEach(function(item) {
                    if (item.hasOwnProperty("RESOURCE"))
                    {
                        var item = this.add.sprite(x*this.config.block_size, y*this.config.block_size, 'resource');
                        item.setDisplaySize(this.config.block_size, this.config.block_size);
                        this.transient_sprites.push(item);
                    }
                }.bind(this));
            }
        }
    },

    update: function()
    {
        if (this.updated === true)
        {
            this.updated = false;
            //console.log(game_state);

            this.transient_sprites.forEach(function(sprite)
            {
                sprite.destroy();
            });
            this.transient_sprites.length = 0;

            if (this.map_created === false) {
                this.map_created = true;
                this.create_map(this.game_state.map);
                this.create_cars(this.game_state.teams);
                this.create_ui(this.game_state.teams);
            }

            this.parse_map(this.game_state.map);

            Object.keys(this.game_state.teams).forEach(id =>
            {
                var team = this.game_state.teams[id];
                var car = this.game_state.cars[id];
                var car_sprite = this.cars[id];

                var x = this.config.block_size*car.x;
                var y = this.config.block_size*car.y;
                car_sprite.setPosition(x, y);
                car_sprite.setTint(parseInt(team.color.replace(/^#/, ''), 16));
                if (car.state.hasOwnProperty("MOVING"))
                {
                    var angle;
                    switch(car.state["MOVING"])
                    {
                        case "NORTH": angle = 0; break;
                        case "SOUTH": angle = 180; break;
                        case "EAST": angle = 90; break;
                        case "WEST": angle = 270; break;
                    }
                    car_sprite.setAngle(angle);
                }
                else
                {
                    car_sprite.setAngle(0);
                    car_sprite.setTintFill(0x555555);
                }
                if (car.collided === true)
                {
                    car_sprite.setTintFill(0xffffff);
                }
                if (car.killed === true)
                {
                    var explosion = this.add.sprite(car.next_x*this.config.block_size, car.next_y*this.config.block_size, 'explosion');
                    explosion.setDisplaySize(this.config.block_size, this.config.block_size);
                    this.transient_sprites.push(explosion);
                }

                this.scoreTexts[id].setText(this.game_state.teams[id].name + ': ' +
                    this.health_to_bar(car.killed ? 0 : car.health, car.max_health) +
                    ' ' + this.game_state.teams[id].score);
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
