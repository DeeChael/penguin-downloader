package com.penguin.downloader.tui

import androidx.compose.runtime.LaunchedEffect
import com.jakewharton.mosaic.layout.fillMaxSize
import com.jakewharton.mosaic.modifier.Modifier
import com.jakewharton.mosaic.runMosaicMain
import com.jakewharton.mosaic.text.buildAnnotatedString
import com.jakewharton.mosaic.ui.Alignment
import com.jakewharton.mosaic.ui.Column
import com.jakewharton.mosaic.ui.Row
import com.jakewharton.mosaic.ui.Text
import kotlinx.coroutines.awaitCancellation

fun main() = runMosaicMain {
    // TODO：来个人实现它吧！
    Row(
        modifier = Modifier.fillMaxSize(),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(
            modifier = Modifier.fillMaxSize(),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text(
                buildAnnotatedString {
                    append("████████..████████.██....██..██████...██.....██.████.██....██\n")
                    append("██.....██.██.......███...██.██....██..██.....██..██..███...██\n")
                    append("██.....██.██.......████..██.██........██.....██..██..████..██\n")
                    append("████████..██████...██.██.██.██...████.██.....██..██..██.██.██\n")
                    append("██........██.......██..████.██....██..██.....██..██..██..████\n")
                    append("██........██.......██...███.██....██..██.....██..██..██...███\n")
                    append("██........████████.██....██..██████....███████..████.██....██\n")
                }
            )
        }
    }
    LaunchedEffect(Unit) {
        awaitCancellation()
    }
}






