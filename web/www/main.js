var Robodrivers = new Phaser.Class({

    Extends: Phaser.Scene,

    initialize: function Robodrivers()
    {
        Phaser.Scene.call(this, { key: 'robodrivers' });
        this.connect();

        this.background;
        this.cars;
        this.scoreTexts;
        this.particles;
        this.emitters;
    },

    game_state: null,
    updated: false,
    map_created: false,
    transient_sprites: [],

    config:
    {
        block_size: 32,
    },

    preload: function()
    {
        //this.load.setBaseURL('http://labs.phaser.io');

        this.load.multiatlas('atlas', 'assets/atlas.json', 'assets');

        this.load.image('red', 'assets/red.png');
        this.load.image('white', 'assets/white.png');
        this.load.image('explosion', 'assets/explosion.png');
    },

    create: function()
    {

        //this.particles = this.add.particles('white');

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
        this.emitters = [];
        Object.keys(teams).forEach(team_id =>
        {
            var team = teams[team_id];
            var car = this.physics.add.sprite(-100000, -100000, 'atlas', 'vehicules/idle.png');
            car.depth = 1;
            car.setDisplaySize(this.config.block_size, this.config.block_size);
            car.setBounce(0.2);
            car.setCollideWorldBounds(true);
            this.cars[team_id] = car;

            /*
            var emitter = this.particles.createEmitter({
                speed: 100,
                scale: { start: 1, end: 0 },
                blendMode: 'ADD',
                depth: 1,
            });
            this.emitters[team_id] = emitter;
            //emitter.startFollow(car);
            */

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

    select_wall: function(map, x, y)
    {
        var wall = 'chipset';
        var max_y = map.cells.length - 1;
        var max_x = map.cells[0].length - 1;

        var is_wall = function(x, y)
        {
            if ((x<0) || (x>max_x)) { return false; };
            if ((y<0) || (y>max_y)) { return false; };
            return map.cells[y][x].block == "WALL";
        }

        var pattern = "";
        pattern += (is_wall(x, y-1)) ? "1" : "0";
        pattern += (is_wall(x+1, y)) ? "1" : "0";
        pattern += (is_wall(x, y+1)) ? "1" : "0";
        pattern += (is_wall(x-1, y)) ? "1" : "0";

        switch (pattern) {
            case '0000': wall = 'chipset'; break;
            case '1111': wall = 'chipset'; break;

            case '1000': wall = 'bottom_end'; break;
            case '0100': wall = 'left_end'; break;
            case '0010': wall = 'top_end'; break;
            case '0001': wall = 'right_end'; break;

            case '1100': wall = 'bottom_left_corner'; break;
            case '0110': wall = 'top_left_corner'; break;
            case '0011': wall = 'top_right_corner'; break;
            case '1001': wall = 'bottom_right_corner'; break;

            case '1010': wall = (x<max_x/2) ? 'left_vertical_wall' : 'right_vertical_wall'; break;
            case '0101': wall = (y<max_y/2) ? 'top_horizontal_wall' : 'bottom_horizontal_wall'; break;

            case '0111': wall = 'top_tee'; break;
            case '1011': wall = 'right_tee'; break;
            case '1101': wall = 'bottom_tee'; break;
            case '1110': wall = 'left_tee'; break;
        }

        return 'walls/' + wall + '.gif';
    },

    select_background: function(map, x, y)
    {
        return 'backgrounds/background_' + Math.floor(Math.random()*3 + 1)  + '.gif';
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
                    var block = this.add.sprite((x+1)*this.config.block_size, (y+1)*this.config.block_size, 'atlas', this.select_wall(map, x, y));
                    block.setDisplaySize(this.config.block_size, this.config.block_size);
                } else if (cell.block == "OPEN")
                {
                    var block = this.add.sprite((x+1)*this.config.block_size, (y+1)*this.config.block_size, 'atlas', this.select_background(map, x, y));
                    block.setDisplaySize(this.config.block_size, this.config.block_size);
                }
                cell.items.forEach(function(item) {
                    if (item === "BASE")
                    {
                        var item = this.add.sprite((x+1)*this.config.block_size, (y+1)*this.config.block_size, 'atlas', 'backgrounds/up_arrow.gif');
                        item.setDisplaySize(this.config.block_size, this.config.block_size);
                    }
                    else if (item.hasOwnProperty("PRODUCER"))
                    {
                        var item = this.add.sprite((x+1)*this.config.block_size, (y+1)*this.config.block_size, 'atlas', 'items/producer.gif');
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
                        var item = this.add.sprite((x+1)*this.config.block_size, (y+1)*this.config.block_size, 'atlas', 'items/barrel5.png');
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

                var x = this.config.block_size*(car.x+1);
                var y = this.config.block_size*(car.y+1);
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
                    car_sprite.setAngle(angle - 90);
                    //this.emitters[id].setPosition(x, y);
                }
                else
                {
                    car_sprite.setAngle(0);
                    car_sprite.setTintFill(0x555555);
                    //this.emitters[id].setPosition(-100000, -100000);
                }
                if (car.collided === true)
                {
                    car_sprite.setTintFill(0xffffff);
                }
                if (car.killed === true)
                {
                    var explosion = this.add.sprite((car.next_x+1)*this.config.block_size, (car.next_y+1)*this.config.block_size, 'explosion');
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
    //backgroundColor: '#ffffff',
    physics: {
        default: 'arcade',
        arcade: {
            gravity: { y: 0 }
        }
    },
    canvas: document.getElementById("gameCanvas"),
    scene: [ Robodrivers ],
}

var game = new Phaser.Game(phaser_config);
