"""
Considerations:
- per address profiles (address = 1 farm)
- Query others farms info
- User inventory of seeds / items
- single leaderboard idea

Logic:
- Start with grass (till into dirt (cost 1 block of time))
- If dirt is not planted in 3-6 hours, turn back to grass (on try to plant, error our for its back to grass. Then set it to grass)

- tilled dirt, add a seed to plant
    different seeds take different amounts of time to grow
    after X blocks, allow them to be harvested'

- Increase farm size for a fee (JUNO, or tokenfactory)


Expand Ideas:
- Use v13 FeeShare
- Use TokenFactory for Seeds / equiptment  (ex: watering can).
"""


profile = "juno1reece"
current_block = 1

from enum import Enum


class Item(Enum):
    GRASS = 0
    DIRT = 1
    SEED = 2  # add different types
    COW = 3

    # when using, only return the name
    def __str__(self):
        return self.name

    def __repr__(self):
        return self.name


# create a 2 key dict to store cooldowns, where the keys are the x y coords
cooldowns = {
    (0, 0): 0,
}

# Write the farm class which stores an array of arrays, each being a plot
class Farm:
    INITIAL_PLOTS = 3

    def __init__(self, address):
        self.address = address
        self.plots = [
            [Item.GRASS for x in range(self.INITIAL_PLOTS)]
            for y in range(self.INITIAL_PLOTS)
        ]

    def get_size(self):
        return len(self.plots)

    def get_plots(self):
        # return self.plots
        # return the plots in order pretty printer
        return "\n" + "\n".join([str(x) for x in self.plots])

    def get_plot(self, x, y):
        return self.plots[x][y]

    def set_plot(self, x, y, value: Item):
        # where value will be the Item type struct
        self.plots[x][y] = value

    def upgrade_size(self, amount: int = 1):
        """
        Initial plot:
        x x x
        x x x
        x x x

        With amount = 1, we add it all around
        x x x x
        x x x x
        x x x x
        x x x x
        Making it now 4x4
        """
        for p in self.plots:
            p.extend([Item.GRASS for x in range(amount)])

        # then append amount of new plots
        self.plots.extend(
            [
                [Item.GRASS for x in range(self.get_size() + amount)]
                for y in range(amount)
            ]
        )

    def till(self, x, y):
        # if grass, turn to dirt
        if self.get_plot(x, y) == Item.GRASS:
            self.set_plot(x, y, Item.DIRT)
            return True
        return False

    def get_cooldown(self, x, y):
        return cooldowns.get((x, y), 0)

    def find_plots_with_type(self, item: Item):
        # return a list of tuples of the x and y coords
        # of the plots with the given item type
        plots = []
        for x in range(self.get_size()):
            for y in range(self.get_size()):
                if self.get_plot(x, y) == item:
                    plots.append((x, y))
        return plots

    def get_all_unique_plot_items(self):
        # return a list of all the unique items
        # on the farm
        items = []
        for x in range(self.get_size()):
            for y in range(self.get_size()):
                item = self.get_plot(x, y)
                if item not in items:
                    items.append(item)
        return items

    def interact(self, x, y):
        # if type is cow, we milk it
        plot = self.get_plot(x, y)
        cooldown = self.get_cooldown(x, y)

        if plot == Item.COW:
            if cooldown > current_block:
                print("Cooldown not ready")
                return False
            print("Milked cow")
            cooldowns[(x, y)] = current_block + 10
            return True

    def __str__(self):
        return str(self.plots)


f = Farm(profile)
plots = f.get_plots()
print(plots)

f.till(0, 0)

f.upgrade_size(1)
print(f.get_plots())

# buy a cow
f.set_plot(0, 1, Item.COW)
print(f.get_plots())

f.interact(0, 1)
f.interact(0, 1)
current_block = 11
f.interact(0, 1)


found_items = f.find_plots_with_type(Item.COW)
print(found_items)

found_items = f.find_plots_with_type(Item.GRASS)
print(found_items)


unique_plot_types = f.get_all_unique_plot_items()
print(unique_plot_types)
