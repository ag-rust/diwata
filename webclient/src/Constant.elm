module Constant exposing (..)

import Html.Attributes exposing (style)
import Util exposing (px)


{-|

    The margin-left for detailed-selected-row
    which is used in the calculation of the DetailedRecord view
    tabs and columns widths
-}
detailedMarginLeft =
    200


detailedSelectedRowStyle =
    style [ ( "margin-left", px detailedMarginLeft ) ]


{-|

    This is used in caculation whether or not the list is scrolled to the bottom
-}
tabRowValueHeight =
    40


tabRowValueStyle =
    style [ ( "height", px tabRowValueHeight ) ]