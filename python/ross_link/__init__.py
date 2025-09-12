from .ross_link import *

__doc__ = ross_link.__doc__
if hasattr(ross_link, "__all__"):
    __all__ = ross_link.__all__


class Schedule(ross_link.Schedule):
    def test_me(self):
        print("Hello from subclass!")


def test():
    print("Hey!")
